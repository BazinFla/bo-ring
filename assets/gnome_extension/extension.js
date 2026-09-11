import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Meta from 'gi://Meta';
import Clutter from 'gi://Clutter';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

const DBUS_SCHEMA = `
<node>
  <interface name="org.boring.WindowTracker">
    <method name="GetActiveWindow">
      <arg type="s" direction="out" name="wm_class"/>
      <arg type="s" direction="out" name="title"/>
      <arg type="i" direction="out" name="pid"/>
    </method>
    <method name="GetWindowUnderCursor">
      <arg type="s" direction="out" name="wm_class"/>
      <arg type="s" direction="out" name="title"/>
      <arg type="i" direction="out" name="pid"/>
    </method>
    <method name="FocusWindowUnderCursor">
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="GetActiveMonitorWorkArea">
      <arg type="i" direction="out" name="x"/>
      <arg type="i" direction="out" name="y"/>
      <arg type="i" direction="out" name="width"/>
      <arg type="i" direction="out" name="height"/>
    </method>
    <signal name="WindowFocused">
      <arg type="s" name="wm_class"/>
      <arg type="s" name="title"/>
      <arg type="i" name="pid"/>
    </signal>
  </interface>
</node>`;

const DESKTOP_IDENTIFIER = 'Global / Desktop';

export default class BoRingWindowTrackerExtension extends Extension {
    enable() {
        this._focusSignalId = 0;
        this._dbusImpl = Gio.DBusExportedObject.wrapJSObject(DBUS_SCHEMA, {
            GetActiveWindow: () => {
                return this._getActiveWindowInfo();
            },
            GetWindowUnderCursor: () => {
                return this._getWindowUnderCursorInfo();
            },
            FocusWindowUnderCursor: () => {
                return this._focusWindowUnderCursor();
            },
            GetActiveMonitorWorkArea: () => {
                return this._getActiveMonitorWorkArea();
            }
        });

        try {
            this._dbusImpl.export(Gio.DBus.session, '/org/boring/WindowTracker');
            console.log('[BoRingExtension] DBus interface /org/boring/WindowTracker exported.');
        } catch (e) {
            console.error(`[BoRingExtension] Failed to export DBus: ${e}`);
        }

        if (global.display) {
            this._focusSignalId = global.display.connect('notify::focus-window', () => {
                const [wmClass, title, pid] = this._getActiveWindowInfo();
                if (this._dbusImpl) {
                    this._dbusImpl.emit_signal('WindowFocused', new GLib.Variant('(ssi)', [wmClass, title, pid]));
                }
            });
        }
    }

    disable() {
        if (this._focusSignalId && global.display) {
            global.display.disconnect(this._focusSignalId);
            this._focusSignalId = 0;
        }

        if (this._dbusImpl) {
            this._dbusImpl.unexport();
            this._dbusImpl = null;
        }
        console.log('[BoRingExtension] Disabled.');
    }

    _isWindowVisibleAndValid(win) {
        if (!win) return false;
        if (win.minimized) return false;
        if (typeof win.is_hidden === 'function' && win.is_hidden()) return false;
        if (typeof win.showing_on_its_workspace === 'function' && !win.showing_on_its_workspace()) return false;
        return true;
    }

    _getActiveWindowInfo() {
        const cursorWinInfo = this._getWindowUnderCursorInfo();
        if (cursorWinInfo && cursorWinInfo[0] !== DESKTOP_IDENTIFIER) {
            return cursorWinInfo;
        }

        const win = global.display ? global.display.focus_window : null;
        if (!win || !this._isWindowVisibleAndValid(win)) {
            return [DESKTOP_IDENTIFIER, '', -1];
        }

        return this._extractWindowDetails(win);
    }

    _getMetaWindowUnderCursor() {
        const [x, y] = this._getPointerCoords();

        // 1. Try Actor Picking via Clutter Stage
        if (global.stage) {
            let actor = null;
            try {
                if (typeof global.stage.get_actor_at_pos === 'function') {
                    actor = global.stage.get_actor_at_pos(Clutter.PickMode.ALL, x, y);
                }
            } catch (e) {
                console.error(`[BoRingExtension] Actor pick error: ${e}`);
            }

            while (actor) {
                let win = null;
                if (typeof actor.get_meta_window === 'function') {
                    win = actor.get_meta_window();
                } else if (actor.meta_window) {
                    win = actor.meta_window;
                } else if (actor._metaWindow) {
                    win = actor._metaWindow;
                }

                if (win && this._isWindowVisibleAndValid(win)) {
                    const details = this._extractWindowDetails(win);
                    const lower = details[0].toLowerCase();
                    if (details[0] !== DESKTOP_IDENTIFIER &&
                        lower !== 'gnome-shell' && 
                        lower !== 'ding' &&
                        lower !== 'bo-ring' &&
                        lower !== 'bo-ring-overlay') {
                        return win;
                    }
                }
                actor = typeof actor.get_parent === 'function' ? actor.get_parent() : null;
            }
        }

        // 2. Fallback: Geometry rect matching via global.display
        if (global.display) {
            try {
                const windows = typeof global.display.get_tab_list === 'function' 
                    ? global.display.get_tab_list(Meta.TabList.NORMAL, null) 
                    : (global.display.list_all_windows ? global.display.list_all_windows() : []);

                for (const win of windows) {
                    if (win && this._isWindowVisibleAndValid(win) && typeof win.get_frame_rect === 'function') {
                        const rect = win.get_frame_rect();
                        if (x >= rect.x && x <= (rect.x + rect.width) &&
                            y >= rect.y && y <= (rect.y + rect.height)) {
                            const details = this._extractWindowDetails(win);
                            const lower = details[0].toLowerCase();
                            if (details[0] !== DESKTOP_IDENTIFIER &&
                                lower !== 'gnome-shell' && 
                                lower !== 'ding' &&
                                lower !== 'bo-ring' &&
                                lower !== 'bo-ring-overlay') {
                                return win;
                            }
                        }
                    }
                }
            } catch (e) {
                console.error(`[BoRingExtension] Geometry window lookup error: ${e}`);
            }
        }

        return null;
    }

    _getWindowUnderCursorInfo() {
        const win = this._getMetaWindowUnderCursor();
        if (win) {
            return this._extractWindowDetails(win);
        }
        return [DESKTOP_IDENTIFIER, '', -1];
    }

    _activateWindow(win) {
        if (!win || !this._isWindowVisibleAndValid(win)) return false;
        try {
            if (Main && typeof Main.activateWindow === 'function') {
                Main.activateWindow(win);
                console.log('[BoRingExtension] Main.activateWindow succeeded');
                return true;
            }
            const time = global.get_current_time ? global.get_current_time() : 0;
            if (typeof win.activate === 'function') {
                win.activate(time);
                console.log('[BoRingExtension] win.activate succeeded');
                return true;
            }
            if (typeof win.focus === 'function') {
                win.focus(time);
                console.log('[BoRingExtension] win.focus succeeded');
                return true;
            }
        } catch (e) {
            console.error(`[BoRingExtension] Activate window error: ${e}`);
        }
        return false;
    }

    _focusWindowUnderCursor() {
        const win = this._getMetaWindowUnderCursor();
        if (win) {
            return this._activateWindow(win);
        }
        return false;
    }

    _getPointerCoords() {
        let x = 0, y = 0;
        try {
            if (typeof global.get_pointer === 'function') {
                const ptr = global.get_pointer();
                x = ptr[0];
                y = ptr[1];
            } else if (global.display && typeof global.display.get_pointer === 'function') {
                const ptr = global.display.get_pointer();
                x = ptr[0];
                y = ptr[1];
            } else if (global.stage && typeof global.stage.get_pointer === 'function') {
                const ptr = global.stage.get_pointer();
                x = ptr[0];
                y = ptr[1];
            }
        } catch (e) {
            console.error(`[BoRingExtension] Pointer error: ${e}`);
        }
        return [x, y];
    }

    _extractWindowDetails(win) {
        if (!win || !this._isWindowVisibleAndValid(win)) {
            return [DESKTOP_IDENTIFIER, '', -1];
        }

        let wmClass = '';
        if (typeof win.get_wm_class === 'function') {
            wmClass = win.get_wm_class() || '';
        }
        if (!wmClass && typeof win.get_gtk_application_id === 'function') {
            wmClass = win.get_gtk_application_id() || '';
        }

        let title = '';
        if (typeof win.get_title === 'function') {
            title = win.get_title() || '';
        }

        let pid = -1;
        if (typeof win.get_pid === 'function') {
            pid = win.get_pid() || -1;
        }

        if (!wmClass && title) {
            wmClass = title;
        }

        if (!wmClass || wmClass.toLowerCase() === 'gnome-shell') {
            wmClass = DESKTOP_IDENTIFIER;
        }

        return [wmClass, title, pid];
    }

    _getActiveMonitorWorkArea() {
        try {
            const [x, y] = this._getPointerCoords();
            let monitorIndex = -1;
            if (global.display) {
                if (typeof global.display.get_monitor_index_for_rect === 'function') {
                    monitorIndex = global.display.get_monitor_index_for_rect(
                        new Meta.Rectangle({ x, y, width: 1, height: 1 })
                    );
                }
                if (monitorIndex < 0 && typeof global.display.get_current_monitor === 'function') {
                    monitorIndex = global.display.get_current_monitor();
                }
            }
            if (monitorIndex < 0) monitorIndex = 0;

            if (Main.layoutManager && typeof Main.layoutManager.getWorkAreaForMonitor === 'function') {
                const workArea = Main.layoutManager.getWorkAreaForMonitor(monitorIndex);
                if (workArea) {
                    return [workArea.x, workArea.y, workArea.width, workArea.height];
                }
            }
            return [0, 0, 1920, 1080];
        } catch (e) {
            console.error(`[BoRingExtension] Error getting active monitor work area: ${e}`);
            return [0, 0, 1920, 1080];
        }
    }
}
