#[macro_use]
extern crate clap;
extern crate rand;
extern crate sdl2;

use clap::{App, Arg};

use rand::distributions::{Distribution, Uniform};

use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::render::{Texture, TextureCreator};
use sdl2::ttf;

use std::thread::sleep;
use std::time::Duration;

mod model;
mod render;

use model::game;
use render::score;
use render::window;

const IMAGES: [&str; 7] = [
    "src/content/normal_board.png",
    "src/content/black_pawn.png",
    "src/content/white_pawn.png",
    "src/content/forbidden_pawn.png",
    "src/content/green_warning.png",
    "src/content/orange_warning.png",
    "src/content/red_warning.png",
];

macro_rules! get_image {
    ($tc:expr, $e:expr) => {
        $tc.load_texture($e).expect("Failed to load image")
    };
}

macro_rules! flush_events {
    ($e:expr, $label:tt) => {
        for event in $e.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break $label,
                _ => {}
            }
        }
    };
}

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
        "long-pro" => game::TypeOfParty::Longpro,
        "pro" => game::TypeOfParty::Pro,
        "standard" => game::TypeOfParty::Standard,
        _ => unreachable!(),
    };

    let nb_of_players = match matches.value_of("NUMBER-OF-PLAYER").unwrap() {
        "0" => 0,
        "1" => 1,
        "2" => 2,
        _ => unreachable!(),
    };
    (nb_of_players, mode)
}

pub fn main() {
    let (nb_of_players, type_of_game) = parse_args_and_fill_structs();
    let (mut game, mut events) = game::Game::new(
        "Gomoku",
        score::WINDOW_LENGTH,
        score::WINDOW_HEIGHT,
        nb_of_players,
        type_of_game,
    )
    .expect("Game intialisation failed");
    let ttf_context = sdl2::ttf::init()
        .map_err(|e| e.to_string())
        .expect("Failes to initialize front displayer");
    let mut font = ttf_context
        .load_font("src/content/OpenSans-Bold.ttf", 128)
        .expect("Failed to load font");
    font.set_style(sdl2::ttf::FontStyle::BOLD);

    let texture_creator: TextureCreator<_> = game.canvas.texture_creator();
    let images: [Texture; 7] = [
        get_image!(texture_creator, IMAGES[0]),
        get_image!(texture_creator, IMAGES[1]),
        get_image!(texture_creator, IMAGES[2]),
        get_image!(texture_creator, IMAGES[3]),
        get_image!(texture_creator, IMAGES[4]),
        get_image!(texture_creator, IMAGES[5]),
        get_image!(texture_creator, IMAGES[6]),
    ];
    let mut rng = rand::thread_rng();
    let choice = Uniform::from(0..20);

    'running: loop {
        if game.actual_player_is_ai().expect("Wrong type of player") {
            let x = choice.sample(&mut rng);
            let y = choice.sample(&mut rng);
            game.change_board_from_input(x, y);
            //            sleep(Duration::new(1, 0));
            flush_events!(events, 'running);
        }
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::MouseButtonDown { x, y, .. } => {
                    game.change_board_from_click(x, y);
                }
                _ => {}
            }
        }
        window::render_window(&mut game, &images, &font);
        sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
