use gtk::*;
use std::process;

use super::super::model::app::*;

pub fn start_gui() {
    if gtk::init().is_err() {
        eprintln!("Failed to initialize GTK Application");
        process::exit(1);
    }

    let app = App::new();

    {
        let gamerule = app.content.gamerule.clone();
        let cpy_button1 = app.content.gamerule1.clone();
        cpy_button1.connect_clicked(move |_| gamerule.set_label("gamerule1"));
    }
    {
        let gamerule = app.content.gamerule.clone();
        let cpy_button2 = app.content.gamerule2.clone();
        cpy_button2.connect_clicked(move |_| gamerule.set_label("gamerule2"));
    }
    {
        let gamerule = app.content.gamerule.clone();
        let cpy_button3 = app.content.gamerule3.clone();
        cpy_button3.connect_clicked(move |_| gamerule.set_label("gamerule3"));
    }
    {
        let gamerule = app.content.gamerule.clone();
        let cpy_button4 = app.content.gamerule4.clone();
        cpy_button4.connect_clicked(move |_| gamerule.set_label("gamerule4"));
    }
    {
        let nbr_player = app.content.nbr_player.clone();
        let button_minus = app.content.player_less.clone();
        button_minus.connect_clicked(move |_| nbr_player.set_label("minus"));
    }
    {
        let nbr_player = app.content.nbr_player.clone();
        let button_add = app.content.player_add.clone();
        button_add.connect_clicked(move |_| nbr_player.set_label("add"));
    }
    {
        let play = app.header.play.clone();
        let window_gameplay = app.window_gameplay.clone();
        let window = app.window.clone();
        play.connect_clicked(move |_| {
            window.destroy();
            window_gameplay.show_all();
        });
    }
    app.window.show_all();
    gtk::main();
}
