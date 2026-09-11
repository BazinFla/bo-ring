# Choix Architecturaux, Invariants & Notes de Conception

Ce document centralise les décisions d'architecture fondamentales, les contraintes non-fonctionnelles, le modèle de sécurité `udev`, les retours d'expérimentation R&D et les pièges techniques identifiés au cours du développement de **Bo-Ring**.

---

## 1. 📐 Directives d'Architecture & Invariants Techniques

Pour préserver la supériorité de Bo-Ring en termes de performances, de réactivité et de légèreté par rapport aux solutions propriétaires (Logitech Options+) ou alternatives (JuhRadial MX, Piper), l'ensemble des développements s'astreint aux principes stricts suivants :

1. **Zéro dépendance lourde (100% Rust Pur)** :
   * Aucun runtime Python, Qt6, GTK4 ou Electron.
   * L'ensemble de la logique (daemon d'arrière-plan, capture `evdev`, injection `uinput`, overlay graphique et configurateur) est compilé dans un **binaire unique autonome**, ultra-rapide et sans dépendances dynamiques complexes.
2. **Budget mémoire strict (< 15 Mo de RAM)** :
   * L'empreinte mémoire d'arrière-plan du daemon en veille doit impérativement rester **inférieure à 15 Mo de RAM** (comparé aux ~150 Mo des outils officiels ou alternatifs).
   * Les caches de textures (`TextureCache`) et futures icônes vectorielles SVG doivent être strictement limités (< 1-2 Mo).
3. **Invariabilité de la sécurité rootless** :
   * L'accès aux périphériques d'entrée repose exclusivement sur `TAG+="uaccess"` scopé par siège de session active `Seat0` via `systemd-logind` (ou groupe `input` en repli).
   * Aucun mode mondial `0666` n'est toléré afin d'éliminer définitivement les risques d'interception ou de keylogger sur le clavier physique principal (voir [Section 3](#3--modèle-de-sécurité-rootless--permissions-udev)).

---

## 2. 🏛️ Organisation Modulaire du Codebase

Le code source est segmenté en couches hautement étanches pour prévenir toute régression :

* **Couche Matérielle (`src/devices/`)** :
  * Découplage complet de la détection et des spécifications matérielles via le registre déclaratif (`DeviceRegistry`, `DeviceModelConfig`).
  * Découverte multi-niveaux de profils JSON (`assets/devices/`, `/usr/share/bo-ring/devices/`, `~/.config/bo-ring/devices/`).
  * **Catalogue Matériel Intégré Clé en Main** : MX Master 4, MX Master 3 & 3S, MX Master 2S, MX Anywhere 3 & 3S, MX Vertical & Lift, M720 Triathlon, MX Ergo Trackball, et Souris Générique 5 boutons.
  * **Moteur Natif Logitech HID++ 2.0 Déclaratif (`hidpp.rs`)** :
    * Interception autonome des commandes matérielles (*CidReporting*) sur `/dev/hidraw*` sans aucun outil externe (Solaar, logiops).
    * Résolution dynamique des CIDs matériels déclarés dans les profils JSON (`0x00C3` pour les gestes pouce, `0x00C4` pour le SmartShift, `0x00D0` pour le pouce M720, `0x00FD` pour le DPI Vertical, `0x00ED` pour la précision Ergo).
    * Neutralisation de la macro clavier d'usine (`Ctrl+Alt+Tab`) et transmission propre des événements dans la boucle de configuration et l'Action Ring.
    * Ré-application automatique du détournement après sortie de veille système (D-Bus `PrepareForSleep`) et reconnexion Bluetooth/USB.
  * Détection agnostique par capacités : tout périphérique exposant `BTN_LEFT` sans touches alphabétiques (`KEY_A`) est surveillé comme souris candidate.
  * Capture concurrente multi-périphériques : chaque souris physique est surveillée dans un thread dédié avec verrouillage exclusif `dev.grab()`, et réémission instantanée des mouvements (`REL_X`, `REL_Y`), molette et clics non remappés via le périphérique virtuel `uinput`.
  * Propagation de l'identité matérielle (`VID:PID` exact et nom) avec chaque événement physique (`MouseInputEvent`), avec priorité absolue aux identifiants matériels (`match_ids`) sur les correspondances textuelles.
* **Couche Plateforme & Compositeurs (`src/platform/`)** :
  * Isole les particularités de chaque environnement de bureau dans des sous-modules étanches :
    * `detect_gnome_window` : Extension GNOME Shell via DBus (`GetWindowUnderCursor`).
    * `detect_kde_window` : Backend `kdotool` (1 ms) avec repli `xprop`.
    * `detect_hyprland_window` : CLI `hyprctl activewindow`.
    * `detect_sway_window` : CLI `swaymsg -t get_tree`.
    * `detect_x11_window` : CLI `xprop -id`.
  * Centralise le fenêtrage transparent overlay (`overlay.rs`), la gestion systemd/autostart (`autostart.rs`), l'extraction de la couleur d'accentuation OS (`accent_color.rs`) et l'émetteur virtuel `uinput` (`input_emitter.rs`).
* **Couche Moteur Radial (`src/ring_menu/`)** :
  * `geometry.rs` : Calcul trigonométrique pur des arcs, rayons dynamiques et zones mortes.
  * `render.rs` : Pipeline de rendu graphique GPU sous `egui`.
  * `state.rs` : Machine à états de l'interaction (survol, sélection, sous-menus emboîtés, hold-to-release, navigation molette).
* **Couche Configuration & Stockage (`src/config/`)** :
  * Single Source of Truth (SSoT) structurée sur disque dans `~/.config/bo-ring/` (`rings/` et `actions/`).
  * Catalogues d'actions thématiques modulaires (`src/config/actions/`) et présets d'applications d'usine (`src/config/presets/`).

---

## 3. 🔒 Modèle de Sécurité Rootless & Permissions udev

Bo-Ring a besoin de trois types d'accès matériels au niveau du noyau Linux :
1. **Lecture des événements souris (`/dev/input/event*`)** : Interception bas niveau des boutons physiques et des mouvements via `evdev`.
2. **Émulation d'entrées (`/dev/uinput`)** : Synthèse de raccourcis clavier et clics virtuels lors de la sélection d'un slot dans l'Action Ring.
3. **Protocole Logitech HID++ 2.0 (`/dev/hidraw*`)** : Détection et détournement natifs (*CidReporting*) des boutons physiques spéciaux (comme le bouton repose-pouce `0x00C3` de la gamme MX Master) pour désactiver la macro matérielle d'usine (`Ctrl+Alt+Tab`) et capturer directement les clics physiques sans aucun utilitaire tiers (Solaar, logiops).

### ⚠️ Pourquoi proscrire formellement `MODE="0666"` et `chmod 0666` ?

Par facilité, les anciennes versions ou certains tutoriels suggéraient d'appliquer des permissions globales `0666` (`rw-rw-rw-`) sur `/dev/uinput` et `/dev/input/event*`. Cette approche introduit des failles majeures :
* **Vecteur Keylogger Critique (`/dev/input/event*`)** : Les nœuds `event*` englobent l'ensemble des périphériques d'entrée de la machine (y compris le **clavier physique principal**). Un mode mondial `0666` permet à **n'importe quel processus local**, conteneur ou application sandboxée d'écouter les frappes du clavier et d'intercepter mots de passe et identifiants.
* **Injection d'Événements Arbitraires (`/dev/uinput`)** : Un accès en écriture libre sur `uinput` permet à un logiciel malveillant local de simuler des séquences de touches sans autorisation.
* **Neutralisation de la protection de session** : Définir `MODE="0666"` avec `TAG+="uaccess"` rend `uaccess` totalement inopérant, le mode mondial court-circuitant l'isolation par utilisateur.

### Architecture des Permissions

Bo-Ring applique le principe du moindre privilège via une architecture à double niveau :
1. **Isolation par Siège de Session (`TAG+="uaccess"`)** :
   * Sur les systèmes avec `systemd-logind` (GNOME, KDE Plasma, Hyprland, Sway, etc.), la directive `TAG+="uaccess"` délègue la gestion des droits à `logind`.
   * `systemd-logind` applique dynamiquement une ACL (`setfacl -m u:<utilisateur>:rw`) uniquement sur la session physique active (`Seat0`).
   * Dès que l'utilisateur verrouille, ferme sa session ou bascule d'utilisateur, les droits sont automatiquement révoqués.
2. **Scoping Matériel Strict** :
   * Filtrage matériel dédié aux périphériques Logitech : `ATTRS{idVendor}=="046d"`.
   * Filtrage dédié aux pointeurs/souris : `ENV{ID_INPUT_MOUSE}=="1"`.
   * Les claviers physiques restent ainsi strictement restreints au système (`0600 root:root`).
3. **Fallback sans logind (Groupe `input`)** :
   * Sur les environnements minimalistes n'utilisant pas `systemd-logind`, les nœuds sont assignés au groupe système `input` avec un mode restreint `0660` (`rw-rw----`), nécessitant l'appartenance explicite de l'utilisateur au groupe `input`.

### Règles udev Déployées (`/etc/udev/rules.d/99-bo-ring-uinput.rules`)

```udev
# udev rules for Bo-Ring (seat-scoped access to uinput, evdev and hidraw for Bluetooth & USB mice)
KERNEL=="uinput", SUBSYSTEM=="misc", OPTIONS+="static_node=uinput", TAG+="uaccess"
KERNEL=="event*", SUBSYSTEM=="input", GROUP="input", MODE="0660", TAG+="uaccess"
KERNEL=="hidraw*", SUBSYSTEM=="hidraw", GROUP="input", MODE="0660", TAG+="uaccess"
```

### Commandes Utiles & Activation à Chaud

* **Rechargement à chaud (sans redémarrer la machine)** :
  ```bash
  sudo udevadm control --reload-rules && sudo udevadm trigger --subsystem-match=input
  ```
* **Application immédiate des ACLs pour la session courante (ex: CachyOS / Arch)** :
  ```bash
  # Accès uinput
  sudo setfacl -m u:"$USER":rw /dev/uinput

  # Accès ciblé aux souris Logitech et souris génériques uniquement (ne touche jamais aux claviers)
  for dev in /dev/input/event*; do
      if udevadm info "$dev" 2>/dev/null | grep -E -q "ID_VENDOR_ID=046d|ID_INPUT_MOUSE=1"; then
          sudo setfacl -m u:"$USER":rw "$dev" 2>/dev/null
      fi
  done
  ```
* **Audit et Vérification des Permissions** :
  ```bash
  # 1. Vérifier que le clavier physique reste bien verrouillé (root:root 0600, pas d'ACL utilisateur) :
  ls -l /dev/input/by-id/ | grep -i kbd

  # 2. Vérifier les ACLs dynamiques sur /dev/uinput et la souris :
  getfacl /dev/uinput
  getfacl /dev/input/eventX  # (remplacer par le numéro de la souris)
  ```

### Références Techniques
* [Spécification des Tags udev & uaccess systemd](https://www.freedesktop.org/software/systemd/man/latest/udev.html#TAGS)
* [Règles udev Solaar (Logitech Unifying / Bolt)](https://github.com/pwr-Solaar/Solaar/blob/master/rules.d/42-logitech-unify-permissions.rules)
* [Projet libratbag / Piper](https://github.com/libratbag/libratbag)

---

## 4. 🔬 Retours d'Expérimentation & R&D

### 📳 Retour Haptique Matériel (Logitech MX Master 4 - Bilan v0.1.0-exp)
Une expérimentation complète a été menée pour implémenter des impulsions haptiques physiques (micro-vibrations lors du survol et de la sélection de slots) via le protocole Logitech HID++ 2.0/4.0 (fonction `0x1F20` sur `/dev/hidraw*`).

* **Causes de mise en pause & défis identifiés** :
  1. **Faux positifs de balayage HID++ (Confusion Clavier/Souris)** : Les claviers Logitech (ex: G513 sur `/dev/hidraw5`) répondent aux requêtes HID++ et exposent la fonction `0x1B04` (Special Keys). Le scanner s'associait par erreur au clavier et lui envoyait les paquets de vibration.
  2. **Complexité d'adressage HID++ (USB vs Bluetooth Direct)** : Sur les dongles USB (C548/Bolt), le pointeur est à `device_idx = 0x01` ou `0x02` (`0xFF` étant le dongle récepteur lui-même). En Bluetooth Direct (`0005:` uhid), l'adresse de la souris est obligatoirement `device_idx = 0xFF`.
  3. **Limites udev sur le bus Bluetooth (`uhid`)** : L'attribut `ATTRS{idVendor}=="046d"` ne s'applique qu'au sysfs USB. Pour les connexions Bluetooth LE (`uhid`), cet attribut n'existe pas dans le chemin parent, laissant les nœuds `/dev/hidraw*` en accès restreint root sans règle udev générique sur `hidraw`.
  4. **Séquence d'activation préalable non documentée** : Même avec un accès en écriture et l'envoi de paquets sur la bonne entité, la souris n'émettait aucune impulsion physique. L'activation de l'actionneur haptique sur MX Master 4 exige vraisemblablement une séquence d'initialisation/enable propriétaire préalable non documentée publiquement par Logitech.
* **Décision** : Le code expérimental a été retiré du tronc principal pour préserver le budget mémoire (< 15 Mo) et la stabilité. Fonctionnalité repoussée en R&D expérimentale pour la `v0.3.0` (nécessite rétro-ingénierie du trafic USB/Bluetooth de Logi Options+).

### 🎯 Ciblage & Focus de Fenêtre sous Wayland
* **Sous GNOME Shell** : Le modèle de sécurité de Wayland interdit à une application cliente de connaître la fenêtre survolée ou d'y injecter directement le focus. L'intégration d'une extension Mutter Clutter dédiée (`bo-ring-window-tracker@flavien`) exposant la méthode DBus `FocusWindowUnderCursor` résout ce problème de manière 100% fiable, permettant à la fenêtre survolée de recevoir le raccourci même sans focus préalable.
* **Sous KDE Plasma** : L'utilisation de `kdotool` (1 ms) interroge KWin instantanément sans la latence ni les contraintes de permissions de D-Bus KWin.
* **Délégation IPC Daemon** : Pour éviter que le configurateur ou l'overlay ne volent le focus lors de l'exécution d'une action, les ordres sont délégués au daemon d'arrière-plan via socket Unix, garantissant une restitution parfaite du focus.

### 👆 Pavé Tactile / Touchpad : Proscription du `dev.grab()` exclusif & Alternatives
* **Contrainte critique Linux** : Sur une souris, Bo-Ring acquiert un verrou exclusif (`dev.grab()`) pour absorber le clic d'origine et empêcher le comportement par défaut de l'OS. Sur un pavé tactile, un `dev.grab()` exclusif est **strictement proscrit** : cela couperait instantanément la transmission du curseur, le défilement fluide à 2 doigts, le zoom et tous les gestes natifs de l'environnement de bureau.
* **Les 3 approches techniques identifiées** :
  1. **Écoute passive evdev (`BTN_TOOL_TRIPLETAP`)** : Le démon ouvre le fichier `/dev/input/event*` du touchpad en mode lecture simple (sans `grab`). Lors d'un contact court (< 250 ms) avec le code `332` (`BTN_TOOL_TRIPLETAP`) sans translation importante, l'Action Ring est ouverte. *Point d'attention* : gérer l'éventuelle concurrence avec l'émulation du clic milieu activée par défaut sur certaines distributions.
  2. **Extension GNOME Shell (Mutter / Clutter)** : Captation du geste tactile à 3 doigts directement dans le compositeur via l'API de Mutter, émettant un signal D-Bus pour afficher l'overlay sans passer par evdev.
  3. **Raccourci externe (Touchegg / Gestes KDE)** : Délégation à un gestionnaire externe appelant directement la commande autonome `bo-ring --ring`.

---

## 5. ⚠️ Notes de Développement & Pièges Connus

### Navigation Navigateur (Page Précédente / Suivante)
* **Recommandation** : Toujours utiliser les combinaisons de touches `Alt + Left` (Précédent) et `Alt + Right` (Suivant) dans la configuration des slots plutôt que les signaux de souris bruts `BTN_BACK` / `BTN_FORWARD`.
* **Raison** : Le signal `BTN_BACK` est généralement capturé par le daemon dans `config.toml` pour déclencher l'ouverture de l'Action Ring (`ShowRingMenu`). Émettre `BTN_BACK` depuis un slot déclenche une ré-interception immédiate par le daemon en boucle au lieu d'atteindre le navigateur web.
