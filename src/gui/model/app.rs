extern crate gtk;
use gtk::*;

use super::*;
use std::process;

pub struct App {
    pub window: Window,
    pub window_gameplay: Window,
    pub header: header::HeaderGamerule,
    pub header_gameplay: header::HeaderGameplay,
    pub content: content::ContentGamerule,
    pub content_gameplay: content::ContentGameplay,
}

impl App {
    pub fn new(stats: std::sync::Arc<game_class::Gamerule>) -> App {
        let window = Window::new(WindowType::Toplevel);
        let window_gameplay = Window::new(WindowType::Toplevel);

        let header = header::HeaderGamerule::new();
        let header_gameplay = header::HeaderGameplay::new();

        let content = content::ContentGamerule::new(stats);
        let content_gameplay = content::ContentGameplay::new();

        let app_name: &str = "Gomoku";
        let app_name_small: &str = "gomoku";
        let size_x: i32 = 1400;
        let size_y: i32 = 1200;

        window.set_titlebar(&header.container);
        window.set_title(app_name);
        window.set_wmclass(app_name_small, app_name);
        window.resize(size_x, size_y);
        window.add(&content.gamemode);
        window.connect_delete_event(move |_, _| {
            main_quit();
            Inhibit(false)
        });

        window_gameplay.set_titlebar(&header_gameplay.container);
        window_gameplay.set_title(app_name);
        window_gameplay.set_wmclass(app_name_small, app_name);
        window_gameplay.resize(size_x, size_y);
        window_gameplay.add(&content_gameplay.grid);
        window_gameplay.connect_delete_event(move |_, _| {
            main_quit();
            Inhibit(false)
        });

        if Window::set_default_icon_from_file("src/gui/content/Black_pawn.png").is_err() {
            eprintln!("Failed to set icon to Application");
            process::exit(1)
        };

        App {
            window,
            window_gameplay,
            header,
            header_gameplay,
            content,
            content_gameplay,
        }
    }
}
