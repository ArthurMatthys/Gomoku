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
    let game = Arc::new(game_class::Game::new());

    let app = app::App::new(&stats);

    {
        let stats = stats.clone();
        let gamerule = app.content.gamerule.clone();
        let cpy_button1 = app.content.gamerule1.clone();
        cpy_button1.connect_clicked(move |_| {
            stats.set_gamerule(0);
            gamerule.set_label(stats.get_gamerule_name());
        });
    }
    {
        let stats = stats.clone();
        let gamerule = app.content.gamerule.clone();
        let cpy_button2 = app.content.gamerule2.clone();
        cpy_button2.connect_clicked(move |_| {
            stats.set_gamerule(1);
            gamerule.set_label(stats.get_gamerule_name());
        });
    }
    {
        let stats = stats.clone();
        let gamerule = app.content.gamerule.clone();
        let cpy_button3 = app.content.gamerule3.clone();
        cpy_button3.connect_clicked(move |_| {
            stats.set_gamerule(2);
            gamerule.set_label(stats.get_gamerule_name());
        });
    }
    {
        let stats = stats.clone();
        let gamerule = app.content.gamerule.clone();
        let cpy_button4 = app.content.gamerule4.clone();
        cpy_button4.connect_clicked(move |_| {
            stats.set_gamerule(3);
            gamerule.set_label(stats.get_gamerule_name());
        });
    }
    {
        let stats = stats.clone();
        let nbr_player = app.content.nbr_player.clone();
        let button_minus = app.content.player_less.clone();
        button_minus.connect_clicked(move |_| {
            stats.set_nb_player(-1);
            nbr_player.set_label(stats.get_player().to_string().as_str());
        });
    }
    {
        let stats = stats.clone();
        let nbr_player = app.content.nbr_player.clone();
        let button_add = app.content.player_add.clone();
        button_add.connect_clicked(move |_| {
            stats.set_nb_player(1);
            nbr_player.set_label(stats.get_player().to_string().as_str());
        });
    }
    {
        let stats = stats.clone();
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
