use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Listener, Manager,
};
use tauri_plugin_decorum::WebviewWindowExt;
use tauri_plugin_log::{Target, TargetKind, RotationStrategy, TimezoneStrategy};

#[cfg(target_os = "macos")]
use tauri::menu::{MenuBuilder, PredefinedMenuItem, SubmenuBuilder};

pub fn run() {
    let mut builder = tauri::Builder::default();

    // Single-instance MUST be registered first in the plugin chain
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }));
    }

    builder
        .plugin(tauri_plugin_decorum::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Warn)
                .targets([
                    Target::new(TargetKind::LogDir { file_name: Some("haven".into()) }),
                    Target::new(TargetKind::Stdout),
                ])
                .max_file_size(5_000_000) // 5MB per file
                .rotation_strategy(RotationStrategy::KeepAll)
                .timezone_strategy(TimezoneStrategy::UseLocal)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Create overlay titlebar: on Windows this hides decorations and injects
            // min/max/close buttons; on macOS it's a no-op since decorations:true
            let main_window = app.get_webview_window("main").unwrap();
            main_window.create_overlay_titlebar().unwrap();

            // --- macOS application menu ---
            #[cfg(target_os = "macos")]
            {
                let app_menu = SubmenuBuilder::new(app, "Haven")
                    .about(None)
                    .separator()
                    .services()
                    .separator()
                    .hide()
                    .hide_others()
                    .show_all()
                    .separator()
                    .quit()
                    .build()?;

                let edit_menu = SubmenuBuilder::new(app, "Edit")
                    .undo()
                    .redo()
                    .separator()
                    .cut()
                    .copy()
                    .paste()
                    .select_all()
                    .build()?;

                let window_menu = SubmenuBuilder::new(app, "Window")
                    .minimize()
                    .maximize()
                    .item(&PredefinedMenuItem::fullscreen(app, None)?)
                    .separator()
                    .close_window()
                    .build()?;

                let menu = MenuBuilder::new(app)
                    .item(&app_menu)
                    .item(&edit_menu)
                    .item(&window_menu)
                    .build()?;

                app.set_menu(menu)?;
            }

            // --- System tray ---
            let show_item = MenuItem::with_id(app, "show", "Show Haven", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit Haven", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let tray = TrayIconBuilder::new()
                .icon(Image::from_bytes(include_bytes!("../icons/tray-default.png"))?)
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .tooltip("Haven")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.unminimize();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.unminimize();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Listen for unread state changes from the frontend
            let tray_handle = tray.clone();
            app.listen("tray-unread-changed", move |event: tauri::Event| {
                let has_unread: bool = serde_json::from_str(event.payload()).unwrap_or(false);
                let icon_bytes: &[u8] = if has_unread {
                    include_bytes!("../icons/tray-unread.png")
                } else {
                    include_bytes!("../icons/tray-default.png")
                };
                if let Ok(icon) = Image::from_bytes(icon_bytes) {
                    let _ = tray_handle.set_icon(Some(icon));
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Haven desktop");
}
