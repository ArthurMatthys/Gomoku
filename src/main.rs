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
//use ia::heuristic;

mod checks;

macro_rules! string_of_index {
    ($line:expr, $col:expr) => {{
        let col: char = std::char::from_u32('A' as u32 + *$col as u32)
            .expect("Could not convert number to char");
        let line = *$line;
        format!("{}{}", col, line)
    }};
}

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
    let images: Vec<Texture> = IMAGES
        .iter()
        .map(|x| get_image!(texture_creator, x))
        .collect::<Vec<Texture>>();
    game.set_changed();

    'running: loop {
        if game.actual_player_is_ai().expect("Wrong type of player") {
            let start = Instant::now();
            let (line, col) = get_ia::get_ia(&mut game);
            let end = Instant::now();
            game.set_player_time(end.duration_since(start));
            println!("{}", string_of_index!(&col, &line));
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
                    keycode: Some(Keycode::H),
                    ..
                } => game.set_capture_pos(),
                Event::KeyDown {
                    keycode: Some(a), ..
                } => println!("{}", a),
                _ => {}
            }
        }
        //        if game.has_changed {
        //            game.history
        //                .iter()
        //                .for_each(|(x, y)| print!("{}//", string_of_index!(x, y)));
        //            println!();
        //        }
        // if game.has_changed {
        //     let mut d = 0i8;
        //     let mut c1 = 0isize;
        //     let mut c2 = 0isize;

        //     let _ = heuristic::first_heuristic_hint(
        //         &mut game.board,
        //         Some(true),
        //         &mut c1,
        //         &mut c2,
        //         &mut d,
        //     );
        //     println!("--------------");
        // }
        if game.check_win() {
            break 'running;
        }
        // ARTHUR
        //    if game.has_changed {
        //        println!("arthur's logic");
        //        heuristic::first_heuristic_hint(&mut game.board, Some(true), &mut 0, &mut 1, &mut 1);
        //    }
        window::render_window(&mut game, &images, &font);
        // DEBUG for check
        // if result { use std::process; println!("GAGNE") ; process::exit(0x0100); }

        sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
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
