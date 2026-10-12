//! macOS: Pagenor's menu tree as the native system menu bar (like InDesign on the Mac).
//! Items dispatch through the same command path as the in-window menus; enablement and check
//! marks are refreshed a few times per second.

use std::collections::HashMap;
use std::str::FromStr;

use muda::accelerator::Accelerator;
use muda::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use pagenor_ui_egui::DesignApp;
use pagenor_ui_egui::menus::{self, AppMenuItem, Item};
use serde_json::Value;

enum Handle {
    Plain(MenuItem),
    Check(CheckMenuItem),
}

pub struct NativeMenu {
    _menu: Menu,
    items: HashMap<String, (String, Value, Handle)>,
    last_refresh: f64,
    recent: Option<Submenu>,
    recent_signature: Value,
    recent_ids: Vec<String>,
    counter: usize,
    /// The interface language the titles are in: the menu is rebuilt when it changes.
    language: String,
}

/// Accelerator for "Cmd+Shift+]" (modifier-less shortcuts stay in the app so typing works).
fn accel(sc: &str) -> Option<Accelerator> {
    if !(sc.contains("Cmd") || sc.contains("Ctrl") || sc.contains("Alt")) {
        return None;
    }
    Accelerator::from_str(&sc.replace("Cmd", "CMD").replace("Alt", "ALT").replace("Shift", "SHIFT").replace("Ctrl", "CTRL")).ok()
}

/// A command's default shortcut, as the in-window menus show it.
fn shortcut(id: &str) -> Option<&'static str> {
    menus::ui_label(id).and_then(|(_, sc)| sc).or_else(|| pagenor_engine::find_command(id).and_then(|c| c.shortcut))
}

impl NativeMenu {
    pub fn install(app: &mut DesignApp) -> Self {
        let menu = Menu::new();
        let mut items = HashMap::new();
        let mut counter = 0usize;
        let mut recent = None;
        // Application menu.
        let app_menu = Submenu::new("Pagenor", true);
        for entry in menus::MAC_APP_MENU {
            let _ = match *entry {
                AppMenuItem::Cmd { id, label, command } => {
                    let sc = shortcut(command).and_then(accel);
                    let item = MenuItem::with_id(id, pagenor_ui_egui::i18n::tr(&app.ui.language, label), true, sc);
                    let appended = app_menu.append(&item);
                    items.insert(id.into(), (command.into(), Value::Null, Handle::Plain(item)));
                    appended
                }
                AppMenuItem::Sep => app_menu.append(&PredefinedMenuItem::separator()),
                AppMenuItem::Services => app_menu.append(&PredefinedMenuItem::services(None)),
                AppMenuItem::Hide => app_menu.append(&PredefinedMenuItem::hide(None)),
                AppMenuItem::HideOthers => app_menu.append(&PredefinedMenuItem::hide_others(None)),
                AppMenuItem::ShowAll => app_menu.append(&PredefinedMenuItem::show_all(None)),
                AppMenuItem::Quit => {
                    // Route Quit through Pagenor's save-all and cancellable recovery flow.
                    let item = MenuItem::with_id("pagenor-quit", pagenor_ui_egui::i18n::tr(&app.ui.language, "Quit Pagenor"), true, accel("Cmd+Q"));
                    let appended = app_menu.append(&item);
                    items.insert("pagenor-quit".into(), ("app.quit".into(), Value::Null, Handle::Plain(item)));
                    appended
                }
            };
        }
        let _ = menu.append(&app_menu);
        for (title, entries) in menus::native_menu_tree_for(app) {
            let sub = Submenu::new(pagenor_ui_egui::i18n::tr(&app.ui.language, title), true);
            build(app, &sub, &entries, &mut items, &mut counter, &mut recent);
            let _ = menu.append(&sub);
        }
        menu.init_for_nsapp();
        app.native_menu = true;
        app.native_shortcuts = items
            .values()
            .filter_map(|(id, p, _)| {
                let sc = shortcut(id)?;
                (p.is_null() && accel(sc).is_some()).then(|| id.clone())
            })
            .collect();
        let mut result = Self {
            _menu: menu,
            items,
            last_refresh: 0.0,
            recent,
            recent_signature: Value::Null,
            recent_ids: Vec::new(),
            counter,
            language: app.ui.language.clone(),
        };
        result.refresh_recent(app);
        result
    }

    pub fn poll(&mut self, app: &mut DesignApp, ctx: &egui::Context) {
        while let Ok(ev) = MenuEvent::receiver().try_recv() {
            if let Some((cmd, params, _)) = self.items.get(ev.id.as_ref()) {
                menus::activate_native(app, ctx, &cmd.clone(), &params.clone());
            }
        }
        // Edit › Interface Language: the system menu bar follows at once, like the panels (the
        // new menu replaces the old one as the application's menu before that one is dropped).
        if app.ui.language != self.language {
            *self = Self::install(app);
        }
        let now = pagenor_ui_egui::now_ms();
        if now - self.last_refresh < 250.0 {
            return;
        }
        self.last_refresh = now;
        self.refresh_recent(app);
        for (cmd, params, h) in self.items.values() {
            let en = menus::native_menu_enabled(app, ctx, cmd);
            match h {
                Handle::Plain(i) => i.set_enabled(en),
                Handle::Check(c) => {
                    c.set_enabled(en);
                    c.set_checked(menus::checked(app, cmd, params).unwrap_or(false));
                }
            }
        }
    }

    fn refresh_recent(&mut self, app: &DesignApp) {
        // Session IDs/unsaved titles also affect activation and the menu label.
        let entries = menus::recent_document_items(app);
        let rows: Vec<_> = entries
            .iter()
            .filter_map(|entry| match entry {
                Item::Cmd { label, params, .. } => Some(serde_json::json!([label, params])),
                _ => None,
            })
            .collect();
        let signature = serde_json::json!([app.ui.language, rows]);
        if signature == self.recent_signature {
            return;
        }
        let Some(sub) = &self.recent else { return };
        for _ in 0..sub.items().len() {
            let _ = sub.remove_at(0);
        }
        for id in self.recent_ids.drain(..) {
            self.items.remove(&id);
        }
        for entry in entries {
            if let Item::Cmd { label, id, params, .. } = entry {
                self.counter += 1;
                let mid = format!("pagenor-recent-{}", self.counter);
                let item = MenuItem::with_id(&mid, pagenor_ui_egui::i18n::tr(&app.ui.language, &label), menus::menu_enabled(app, &id), None);
                let _ = sub.append(&item);
                self.recent_ids.push(mid.clone());
                self.items.insert(mid, (id, params, Handle::Plain(item)));
            }
        }
        self.recent_signature = signature;
    }
}

fn build(
    app: &DesignApp,
    parent: &Submenu,
    entries: &[Item],
    items: &mut HashMap<String, (String, Value, Handle)>,
    counter: &mut usize,
    recent: &mut Option<Submenu>,
) {
    for e in entries {
        match e {
            Item::Sep => {
                let _ = parent.append(&PredefinedMenuItem::separator());
            }
            Item::Sub(label, children) => {
                let sub = Submenu::new(pagenor_ui_egui::i18n::tr(&app.ui.language, label), true);
                if label == "Open Recent" {
                    *recent = Some(sub.clone());
                }
                build(app, &sub, children, items, counter, recent);
                let _ = parent.append(&sub);
            }
            Item::Cmd { label, id, params, shortcut } => {
                *counter += 1;
                let mid = format!("pagenor-{counter}");
                let sc = if params.is_null() { shortcut.and_then(accel) } else { None };
                let label = pagenor_ui_egui::i18n::tr(&app.ui.language, label);
                let handle = if menus::checked(app, id, params).is_some() {
                    let c = CheckMenuItem::with_id(mid.clone(), label, true, false, sc);
                    let _ = parent.append(&c);
                    Handle::Check(c)
                } else {
                    let i = MenuItem::with_id(mid.clone(), label, true, sc);
                    let _ = parent.append(&i);
                    Handle::Plain(i)
                };
                items.insert(mid, (id.clone(), params.clone(), handle));
            }
        }
    }
}
