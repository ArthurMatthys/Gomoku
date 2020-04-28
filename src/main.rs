mod initialization;

use initialization::game;
use initialization::textures;
use initialization::game::TypeOfParty;

// pub fn run(game:game::Game, textures:initialization::textures::Textures) -> Result<(), String> {

// }

pub fn main() -> Result<(), String> {
    println!();
    
    // Initalize structs and parse input
    let mut game = game::Game::new("Gomoku", 800, 600, 2, TypeOfParty::Unset)?;
    let mut textures = textures::Textures::new(&game)?;

    // run(game, textures)?;

    Ok(())
}