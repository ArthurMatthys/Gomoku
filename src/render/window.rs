extern crate sdl2;

use sdl2::render::Texture;

use super::super::model::game;

use super::board;
use super::score;

pub fn render_window(game: &mut game::Game, images: &Vec<Texture>, font: &sdl2::ttf::Font) -> () {
    if !game.has_changed {
        return;
    }
    game.canvas.clear();
    score::render_score(game, font);
    board::render_board(game, images);
    game.canvas.present();
    //   let player = game.get_actual_player_mutable();
    //   player.test_forbidden();
    game.has_changed = false;
}
