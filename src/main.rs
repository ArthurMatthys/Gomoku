extern crate sdl2;
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::render::{Texture, TextureCreator};
use std::thread::sleep;
use std::time::Duration;

mod model;
mod render;
use model::game;
use render::board;

const IMAGES: [&str; 7] = [
    "src/content/normal_board.png",
    "src/content/black_pawn.png",
    "src/content/white_pawn.png",
    "src/content/forbidden_pawn.png",
    "src/content/green_warning.png",
    "src/content/orange_warning.png",
    "src/content/red_warning.png",
];

pub fn main() {
    let (mut game, mut events) = game::Game::new("Gomoku", 1400, 1000, 2, game::TypeOfParty::Unset)
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
        board::render_board(&mut game, &images);
        sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
