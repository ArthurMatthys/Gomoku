use crate::initialization::resource::TextureManager;
use sdl2::render::{Texture};
use std::rc::Rc;

// use sdl2::render::{TextureCreator, Texture};
// use sdl2::image::LoadTexture;
// use  sdl2::render::Texture;
// use  sdl2::render::TextureValueError;
use  super::game::Game;

const Images: [&str; 3] = ["../files/black.png", "../files/white.png", "../files/board.png"];

pub struct Textures<'l> {
    // Original board texture
    pub orig_board: Rc<Texture<'l>>,
    pub black_player_texture: Rc<Texture<'l>>,
    pub white_player_texture: Rc<Texture<'l>>,
}



impl<'l> Textures<'l> {

    pub fn new(game: &Game) -> Result<Textures, String> {
        let canva = &game.get_canvas();
        let texture_creator = canva.texture_creator();
        let mut texture_manager = TextureManager::new(&texture_creator);

        let board_texture = texture_manager.load("../files/board.png")?;
        let white_player_texture = texture_manager.load("../files/white.png")?;
        let black_player_texture = texture_manager.load("../files/black.png")?;

        let lala = Ok(Textures {
            orig_board: board_texture,
            white_player_texture: white_player_texture,
            black_player_texture: black_player_texture,
            // curr_board: board_texture,
        });
        lala
    }

    // pub fn draw(&self, canvas: &mut Canvas<Window>) -> Result<(), Box<dyn Error>> {
    //     canvas.copy(&self.texture, None, 
    //         Rect::new(self.pos.x as i32, self.pos.y as i32, 
    //                   self.rect.width(), self.rect.height()))?;

    //     Ok(())
    // }

}