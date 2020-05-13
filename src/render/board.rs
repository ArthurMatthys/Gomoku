extern crate sdl2;

use sdl2::rect::Rect;
use sdl2::render::Texture;

use super::super::model::game;

pub const SIZE_BOARD: usize = 19;
pub const SQUARE_SIZE: usize = 51;

macro_rules! rect {
    ($x:expr, $y:expr) => {
        Rect::new(
            (SQUARE_SIZE * $x) as i32,
            (SQUARE_SIZE * $y) as i32,
            SQUARE_SIZE as u32,
            SQUARE_SIZE as u32,
        )
    };
}

pub fn render_board(game: &mut game::Game, images: &Vec<Texture>) -> () {
    for x in 0..20 {
        for y in 0..20 {
            if x + y == 0 {
                game.canvas
                    .copy(&images[45], None, rect!(x, y))
                    .expect("failed to render image");
                continue;
            }
            if x == 0 {
                game.canvas
                    .copy(&images[7 + 19 + y - 1], None, rect!(x, y))
                    .expect("failed to render image");
                continue;
            }
            if y == 0 {
                game.canvas
                    .copy(&images[7 + x - 1], None, rect!(x, y))
                    .expect("failed to render image");
                continue;
            }
            let new_x = x - 1;
            let new_y = y - 1;
            match game.board[new_x][new_y] {
                None => {
                    if game.is_forbidden_from_coord(new_x, new_y) {
                        game.canvas
                            .copy(&images[3], None, rect!(x, y))
                            .expect("failed to render image");
                    } else if game.is_capture_from_coord(new_x, new_y) {
                        game.canvas
                            .copy(&images[4], None, rect!(x, y))
                            .expect("failed to render image");
                    } else {
                        game.canvas
                            .copy(&images[0], None, rect!(x, y))
                            .expect("failed to render image");
                    }
                    ()
                }
                Some(false) => {
                    game.canvas
                        .copy(&images[1], None, rect!(x, y))
                        .expect("failed to render image");
                    ()
                }
                Some(true) => {
                    game.canvas
                        .copy(&images[2], None, rect!(x, y))
                        .expect("failed to render image");
                    ()
                }
            }
        }
    }
    ()
}
