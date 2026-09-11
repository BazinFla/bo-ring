# 🗺️ Roadmap Bo-Ring (Feuille de Route)

Ce document détaille la vision future, les prochains jalons de version, le backlog d'idées et la matrice de compatibilité du projet **Bo-Ring**.

> [!NOTE]
> * Ce document répertorie **exclusivement les développements à venir et en cours**.
> * Tout ce qui a déjà été développé et livré (y compris les fonctionnalités de la version `v0.2.0` en cours) est consigné en détail dans le [CHANGELOG.md](file:///mnt/c2e1eaa1-0696-43b3-bd52-bd09fefd13ed/Codes/BoRing/bo-ring/CHANGELOG.md).
> * Pour les choix techniques, invariants de performance, sécurité `udev` et retours de R&D, consultez [architecture.md](docs/architecture.md).

---

## 🚀 Prochain Jalon : v0.2.0 — Extensions Compositeurs Avancés & Écosystème

### 1. Extensions Compositeurs Wayland & Layer-Shell (Sway / Hyprland / COSMIC)
Grâce à la couche modulaire unifiée `src/platform/`, la v0.2.0 apporte une intégration native pour les compositeurs Wayland avancés :
* **[ ] Support Native `wlr-layer-shell` (`src/platform/overlay.rs`)** : Intégration du protocole `wlr-layer-shell` (via `smithay-client-toolkit` et `ext-layer-shell-v1`) pour Sway, Hyprland, Wayfire et COSMIC Desktop, permettant un rendu overlay 100% natif sans aucune dépendance à XWayland.
* **[ ] Gestion Multi-Écrans Native (`wlr-output-management`)** : Détection exacte des moniteurs et espaces de travail actifs de manière native Wayland.
* **[ ] Événements Pointeur Natifs (`wl_pointer`)** : Capture et positionnement précis du curseur via les protocoles Wayland officiels.

### 2. Périphériques & Détection Matérielle
* **[ ] Fournisseur unifié de batterie** : Standardisation du suivi de charge pour toutes les souris Bluetooth via `/sys/class/power_supply/`, tout en conservant les protocoles propriétaires (HID++, etc.) comme plugins modulaires.
* **[ ] Hotplug Événementiel via `inotify` / `udev`** : Remplacement de la boucle de scan par intervalle (actuellement 1.5s dans `main.rs`) par une écoute réactive `inotify` sur `/dev/input/` (via `libc`), avec temporisation/retry pour l'application des règles `uaccess` par udev, garantissant une capture instantanée dès l'insertion physique sans polling continu.
* **[ ] Mappings de boutons par périphérique (Multi-souris)** : Permettre d'associer des actions spécifiques par modèle de souris (identifié par `VID:PID`) en complément de la table globale `config.buttons`, afin que deux souris connectées simultanément puissent posséder des profils de raccourcis indépendants.

### 3. Rendu d'icônes vectorielles SVG & Packs d'icônes système
* **[ ] Support des fichiers SVG & Thèmes système** : Rendu net des icônes vectorielles `.svg` personnalisées et résolution automatique des noms d'icônes standards Freedesktop (`media-playback-start`, `edit-copy`, etc.) via les thèmes installés (Papirus, Breeze, etc.).
* **[ ] Intégration Rust** : Utilisation de `resvg` / `usvg` pour générer directement des textures 2D `egui::TextureHandle`.
* **[ ] Mise en cache stricte** : Empreinte mémoire vectorielle maintenue sous ~1-2 Mo.

### 4. Bibliothèque & Partage de Présets
* **[ ] Import / Export Communautaire** : Système d'échange de présets d'Action Ring ou de sous-menus d'actions thématiques (productivité, multimédia, développement, graphisme) pour faciliter le partage entre utilisateurs.

### 5. Mode Gaming
* **[ ] Désactivation temporaire** : Option pour suspendre l'Action Ring et les réassignations de boutons lors du lancement de jeux afin d'éviter tout conflit avec les commandes in-game.

---

## 🔬 Jalons Futurs : v0.3.0 & R&D Avancée

### 1. 📳 Retour Haptique Matériel (Logitech MX Master 4)
* **Objectif** : Émission de micro-impulsions haptiques (retours tactiles physiques) lors du survol et du clic sur un slot.
* **Statut R&D** : Nécessite une phase de rétro-ingénierie du trafic HID++ USB/Bluetooth de Logi Options+ pour identifier la séquence d'activation du moteur physique (voir bilan complet dans [architecture.md](docs/architecture.md#retour-haptique-matériel-logitech-mx-master-4---bilan-v010-exp)).

### 2. Easy-Switch Logiciel (Multi-Hôtes)
* Bascule instantanée de l'ordinateur contrôlé par la souris (1, 2 ou 3) par voie logicielle via requêtes HID++, avec affichage en temps réel du nom des machines appairées.

### 3. BoFlow (Partage Réseau Local Inter-Machines)
* Équivalent open source de Logitech Flow : basculement fluide du curseur d'un écran d'ordinateur à un autre sur le réseau local avec partage du presse-papier chiffré de bout en bout (clés X25519 + AES-256-GCM sans dépendance au cloud).

### 4. Réglages Matériels Avancés (HID++)
* Ajustement dynamique des DPI, configuration du débrayage automatique de la molette SmartShift et lecture télémétrique poussée.

### 5. Moteur de Macros Séquentielles
* Éditeur de séquences de touches programmables avec délais configurables (combinaisons complexes et frappes différées).

### 6. 👆 Support Pavé Tactile / Touchpad (Tap à 3 doigts)
* Déclenchement de l'Action Ring via un contact court à 3 doigts (`BTN_TOOL_TRIPLETAP`) ou via geste GNOME Shell / Clutter, sans verrouillage exclusif (`dev.grab`) afin de préserver l'intégralité des gestes et du défilement natif de l'OS (voir analyse dans [architecture.md](docs/architecture.md#pavé-tactile--proscription-du-devgrab-exclusif)).

---

## 💡 Idées de Slots & Actions Contextuelles (Backlog)

* **Pipette de couleur instantanée** : Récupérer le code couleur hexadécimal/RGB exactement sous le curseur et l'afficher au centre du ring tout en le copiant dans le presse-papier.
* **Action sur texte sélectionné** : Détection du texte en surbrillance sous le curseur pour déclencher une action contextuelle (recherche web, mémo rapide, traduction, exécution script).
* **Traitement du fichier sous curseur** : Récupération du chemin du fichier survolé dans le gestionnaire de fichiers pour le passer en paramètre à un script ou l'ouvrir dans une application dédiée.

---

## 🧪 Matrice de Validation & Tests Multi-Distributions (À Valider)

- [ ] **Fedora 44** :
  - [ ] KDE Plasma
  - [ ] Cinnamon
  - [ ] Xfce
  - [ ] MATE + Compiz
  - [ ] LXQt / LXDE
  - [ ] Sway
  - [ ] COSMIC Desktop
- [ ] **Debian 13** :
  - [ ] GNOME 50 (Wayland)
  - [ ] KDE Plasma
- [ ] **Ubuntu** (LTS & Current)
- [ ] **openSUSE** (Tumbleweed / Leap)
- [ ] **CachyOS / Arch** :
  - [ ] Hyprland (dernière version)
  - [ ] Sway
