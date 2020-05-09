// use super::super::model::point;
// use rand::distributions::{Distribution, Uniform};
// use rand::thread_rng;
use rand::seq::SliceRandom;
use super::super::model::game;
use super::super::checks::capture;
use super::super::checks::search_space;

pub fn dumb_ia(game: &mut game::Game, rng: &mut rand::prelude::ThreadRng) -> usize {
    let available_positions = search_space::search_space(game);
    *available_positions.choose(rng).expect("Error in random extraction") as usize
}

// Need to take history into account, found some issue with double_three
pub fn get_ia(game: &mut game::Game) -> usize {
    // copy get_ia
    let mut rng = rand::thread_rng();

    match game.move_number {
        0 => { println!("{}", "passé dans 0") ; game.move_number += 1 ; 180 },
        2 => { 
            println!("{}", "passé dans 1") ;
            game.move_number += 1 ;
                match game.type_of_party {
                    game::TypeOfParty::Pro =>  (180 + (capture::DIRS.choose(&mut rng)
                                                        .expect("Error in random extraction") * 3)) as usize,
                    game::TypeOfParty::Longpro => (180 + (capture::DIRS.choose(&mut rng)
                                                        .expect("Error in random extraction") * 4)) as usize,
                    game::TypeOfParty::Standard => (180 + (capture::DIRS.choose(&mut rng)
                                                        .expect("Error in random extraction"))) as usize,
                }
            },
        _=> { game.move_number += 1 ; dumb_ia(game, &mut rng) },
    }
}
