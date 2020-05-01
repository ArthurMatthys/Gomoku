extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::rect::Rect;

use super::super::model::game;

use super::board;

const SIZE_BOARDGAME: u32 = (board::SIZE_BOARD * board::SQUARE_SIZE) as u32;
const SIZE_SCORE: u32 = WINDOW_LENGTH - SIZE_BOARDGAME;

pub const WINDOW_LENGTH: u32 = 1400;
pub const WINDOW_HEIGHT: u32 = 961;

macro_rules! rect_score {
    ($x:expr, $y:expr, $l:expr, $h:expr) => {
        Rect::new((SIZE_BOARDGAME + $x) as i32, $y, $l, $h)
    };
}

macro_rules! render_text {
    ($font:expr, $tc:expr, $msg:expr,$cv:expr, $x:expr, $y:expr, $l:expr, $h:expr) => {{
        let surface = $font
            .render($msg)
            .blended(Color::RGB(0, 0, 0))
            .expect("Could not render a color");
        let texture = $tc
            .create_texture_from_surface(&surface)
            .expect("Could not render a surface");
        $cv.copy(&texture, None, rect_score!($x, $y, $l, $h))
            .expect("Could not render texture");
    }};
}

pub fn render_score(game: &mut game::Game, font: &sdl2::ttf::Font) -> () {
    let tc = game.canvas.texture_creator();
    game.canvas.set_draw_color(Color::RGB(255, 255, 255));
    game.canvas
        .fill_rect(rect_score!(0, 0, SIZE_SCORE, WINDOW_HEIGHT))
        .expect("Failed to render white rect");
    render_text!(
        font,
        tc,
        game.party_to_string(),
        game.canvas,
        0,
        0,
        SIZE_SCORE,
        100
    );
    render_text!(
        font,
        tc,
        game.get_player1(),
        game.canvas,
        0,
        100,
        SIZE_SCORE / 2 - 20,
        100
    );
    render_text!(
        font,
        tc,
        game.get_player2(),
        game.canvas,
        SIZE_SCORE / 2 + 20,
        100,
        SIZE_SCORE / 2 - 20,
        100
    );
    if game.result {
        let winner = match game.history.len() % 2 {
            0 => "Player 2 won",
            1 => "Player 1 won",
            _ => unreachable!(),
        };
        render_text!(font, tc, winner, game.canvas, 0, 200, SIZE_SCORE, 100)
    } else {
        render_text!(
            font,
            tc,
            game.get_player_turn_display(),
            game.canvas,
            0,
            200,
            SIZE_SCORE,
            100
        )
    };
}
