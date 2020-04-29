extern crate sdl2;
use sdl2::rect::Rect;
use sdl2::render::Texture;

use super::super::model::game;
use super::super::model::player;

const SIZE_BOARD: usize = 19;
const SQUARE_SIZE: usize = 51;

fn get_rect(x: usize, y: usize) -> Rect {
    Rect::new(
        (SQUARE_SIZE * x) as i32,
        (SQUARE_SIZE * y) as i32,
        SQUARE_SIZE as u32,
        SQUARE_SIZE as u32,
    )
}

pub fn render_board(game: &mut game::Game, images: &[Texture; 7]) -> () {
    //    let (mut canvas, mut player) = game.get_player_canvas();
    game.canvas.clear();
    for x in 0..19 {
        for y in 0..19 {
            match game.board[x * SIZE_BOARD + y] {
                None => {
                    if game
                        .get_actual_player()
                        .forbidden
                        .iter()
                        .any(|&point| point.is_equal(x, y))
                    {
                        game.canvas
                            .copy(&images[3], None, get_rect(x, y))
                            .expect("failed to render image");
                    } else {
                        game.canvas
                            .copy(&images[0], None, get_rect(x, y))
                            .expect("failed to render image");
                    }
                    ()
                }
                Some(false) => {
                    game.canvas
                        .copy(&images[1], None, get_rect(x, y))
                        .expect("failed to render image");
                    ()
                }
                Some(true) => {
                    game.canvas
                        .copy(&images[2], None, get_rect(x, y))
                        .expect("failed to render image");
                    ()
                }
            }
        }
    }
    game.canvas.present();
    let player = game.get_actual_player_mutable();
    player.test_forbidden();
    ()
}
