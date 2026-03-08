use tauri::Manager;
use tauri_plugin_decorum::WebviewWindowExt;

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

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Haven desktop");
}
