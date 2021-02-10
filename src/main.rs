#[macro_use]
extern crate clap;
extern crate rand;
extern crate sdl2;

use clap::{App, Arg};

use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::render::{Texture, TextureCreator};
use std::thread::sleep;

use std::time::Duration;
use std::time::Instant;

mod model;
use model::game;
use model::player;

mod render;
use render::score;
use render::window;

mod ia;
use ia::get_ia;
use ia::zobrist;
//use ia::heuristic;
mod checks;

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

const DEPTH_MAX: i8 = 10;

//macro_rules! string_of_index {
//    ($line:expr, $col:expr) => {{
//        let col: char = std::char::from_u32('A' as u32 + *$col as u32)
//            .expect("Could not convert number to char");
//        let line = *$line;
//        format!("{}{}", col, line)
//    }};
//}

const IMAGES: [&str; 46] = [
    "src/content/normal_board.png",
    "src/content/black_pawn.png",
    "src/content/white_pawn.png",
    "src/content/forbidden_pawn.png",
    "src/content/green_warning.png",
    "src/content/orange_warning.png",
    "src/content/red_warning.png",
    "src/content/letters/A.png",
    "src/content/letters/B.png",
    "src/content/letters/C.png",
    "src/content/letters/D.png",
    "src/content/letters/E.png",
    "src/content/letters/F.png",
    "src/content/letters/G.png",
    "src/content/letters/H.png",
    "src/content/letters/I.png",
    "src/content/letters/J.png",
    "src/content/letters/K.png",
    "src/content/letters/L.png",
    "src/content/letters/M.png",
    "src/content/letters/N.png",
    "src/content/letters/O.png",
    "src/content/letters/P.png",
    "src/content/letters/Q.png",
    "src/content/letters/R.png",
    "src/content/letters/S.png",
    "src/content/nb/00.png",
    "src/content/nb/01.png",
    "src/content/nb/02.png",
    "src/content/nb/03.png",
    "src/content/nb/04.png",
    "src/content/nb/05.png",
    "src/content/nb/06.png",
    "src/content/nb/07.png",
    "src/content/nb/08.png",
    "src/content/nb/09.png",
    "src/content/nb/10.png",
    "src/content/nb/11.png",
    "src/content/nb/12.png",
    "src/content/nb/13.png",
    "src/content/nb/14.png",
    "src/content/nb/15.png",
    "src/content/nb/16.png",
    "src/content/nb/17.png",
    "src/content/nb/18.png",
    "src/content/empty.png",
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
                .help("What mode to run the program in")
                .index(1)
                .possible_values(&["long-pro", "pro", "standard"])
                .required(true),
        )
        .arg(
            Arg::with_name("NUMBER-OF-PLAYER")
                .help("Number of Human Players in the game")
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
    let images: Vec<Texture> = IMAGES
        .iter()
        .map(|x| get_image!(texture_creator, x))
        .collect::<Vec<Texture>>();
    game.set_changed();
    let ztable = zobrist::init_zboard();

    let start_game = Instant::now();
    'running: loop {
        if game.actual_player_is_ai().expect("Wrong type of player") {
            zobrist::clear_tt();
            let start = Instant::now();
            let (line, col) = get_ia::get_ia(&mut game, &ztable, &DEPTH_MAX, &start);
            let end = Instant::now();
            game.set_player_time(end.duration_since(start));
            game.change_board_from_input(line, col);
            flush_events!(events, 'running);
            //    sleep(Duration::new(1, 0000000));
        }
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::MouseButtonDown { x, y, .. } => {
                    // game.change_board_from_click(x, y);
                    game.change_board_from_click(y, x);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Backspace),
                    ..
                } => {
                    game.clear_last_move();
                    if game.players.0.player_type == player::TypeOfPlayer::Robot
                        || game.players.1.player_type == player::TypeOfPlayer::Robot
                    {
                        game.clear_last_move();
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::T),
                    ..
                } => game.set_capture_pos(),
                Event::KeyDown {
                    keycode: Some(Keycode::H),
                    ..
                } => {
                    let start = Instant::now();
                    let (line, col) = get_ia::get_ia(&mut game, &ztable, &4, &start);
                    game.set_best_move(line, col);
                }
                Event::KeyDown {
                    keycode: Some(a), ..
                } => println!("{}", a),
                _ => {}
            }
        }
        //        if game.history.len() == 1 {
        //            let (dx, dy) = (1isize, -1isize);
        //            let (new_x, new_y) = game.history[0];
        //            game.change_board_from_input(
        //                (new_x as isize + dx) as usize,
        //                (new_y as isize + dy) as usize,
        //            );
        //        }

        if game.check_win() {
            break 'running;
        }
        window::render_window(&mut game, &images, &font);

        sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    let end_game = Instant::now();
    let delta = end_game.duration_since(start_game);
    println!("time : {}.{}", delta.as_secs(), delta.subsec_millis());
    if game.instant_win {
        window::render_window(&mut game, &images, &font);
        'ending: loop {
            for event in events.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'ending,
                    _ => {}
                }
            }
        }
    }
}
