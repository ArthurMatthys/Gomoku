use gtk::*;
use std::process;
use std::sync::Arc;

use super::super::model::*;

pub fn start_gui() -> () {
    if gtk::init().is_err() {
        eprintln!("Failed to initialize GTK Application");
        process::exit(1);
    }
    let stats: std::sync::Arc<game_class::Gamerule> = Arc::new(game_class::Gamerule::new(1, 0));
    let stats_clone = stats.clone();
    let game = Arc::new(game_class::Game::new());

    let app = app::App::new(stats);

    {
        let stats = stats_clone.clone();
        let game = game.clone();
        let play = app.header.play.clone();
        let window_gameplay = app.window_gameplay.clone();
        let window = app.window.clone();
        play.connect_clicked(move |_| {
            window.destroy();
            window_gameplay.show_all();
            game.set_info(stats.get_player(), stats.get_gamerule());
            println!("Game created !");
        });
    }
    app.window.show_all();
    gtk::main();
}
