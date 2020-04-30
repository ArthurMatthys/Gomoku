#[macro_use]
extern crate clap;
extern crate sdl2;

use clap::{App, Arg};
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::render::{Texture, TextureCreator};
use std::thread::sleep;
use std::time::Duration;

mod model;
mod render;
mod checks;
use model::game;
use render::board;
use checks::after_turn_check;

const IMAGES: [&str; 7] = [
    "src/content/normal_board.png",
    "src/content/black_pawn.png",
    "src/content/white_pawn.png",
    "src/content/forbidden_pawn.png",
    "src/content/green_warning.png",
    "src/content/orange_warning.png",
    "src/content/red_warning.png",
];

fn parse_args_and_fill_structs() -> (usize, game::TypeOfParty) {
    let matches = App::new("Gomoku")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Implementation of Gomoku as a school's project")
        .arg(
            Arg::with_name("MODE")
                .about("What mode to run the program in")
                .index(1)
                .possible_values(&["long-pro", "pro", "standard"])
                .required(true),
        )
        .arg(
            Arg::with_name("NUMBER-OF-PLAYER")
                .about("Number of Human Players in the game")
                .index(2)
                .possible_values(&["0", "1", "2"])
                .required(true),
        )
        .get_matches();
    
    let mode = match matches.value_of("MODE").unwrap() {
        "long-pro" => { game::TypeOfParty::Longpro }
        "pro" => { game::TypeOfParty::Pro }
        "standard" => { game::TypeOfParty::Standard }
        _ => unreachable!(),
    };

    let nb_of_players = match matches.value_of("NUMBER-OF-PLAYER").unwrap() {
        "0" => { 0 }
        "1" => { 1 }
        "2" => { 2 }
        _ => unreachable!(),
    };
    (nb_of_players,mode)
}

pub fn main() {
    let (nb_of_players, type_of_game) = parse_args_and_fill_structs();
    let (mut game, mut events) = game::Game::new("Gomoku", 1400, 1000, nb_of_players, type_of_game)
        .expect("Game intialisation failed");

    let texture_creator: TextureCreator<_> = game.canvas.texture_creator();
    let images: [Texture; 7] = [
        texture_creator
            .load_texture(IMAGES[0])
            .expect("Failed to load image"),
        texture_creator
            .load_texture(IMAGES[1])
            .expect("Failed to load image"),
        texture_creator
            .load_texture(IMAGES[2])
            .expect("Failed to load image"),
        texture_creator
            .load_texture(IMAGES[3])
            .expect("Failed to load image"),
        texture_creator
            .load_texture(IMAGES[4])
            .expect("Failed to load image"),
        texture_creator
            .load_texture(IMAGES[5])
            .expect("Failed to load image"),
        texture_creator
            .load_texture(IMAGES[6])
            .expect("Failed to load image"),
    ];

    'running: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::MouseButtonDown { x, y, .. } => {
                    game.change_board(x, y);
                }
                _ => {}
            }
        }
        let result = after_turn_check::check_winner(&game);
        // DEBUG for check
        // if result { use std::process; println!("GAGNE") ; process::exit(0x0100); }

        board::render_board(&mut game, &images);
        sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
