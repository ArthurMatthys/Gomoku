// use super::super::model::point;
// use rand::distributions::{Distribution, Uniform};
// use rand::thread_rng;
use super::super::checks::capture;
use super::super::checks::search_space;
use super::super::model::game;
use rand::seq::SliceRandom;

pub fn dumb_ia(game: &mut game::Game, rng: &mut rand::prelude::ThreadRng) -> usize {
    let available_positions = search_space::search_space(game);
    *available_positions
        .choose(rng)
        .expect("Error in random extraction") as usize
}

// Aim of the function :
// Bitwise transpose Bool option board to bitArray (2 bits == 1 Bool option index)
fn transpose_board_to_bitboard(game: &mut game::Game, bitboard: &mut [u64; 12]) -> () {
    game.board.iter().enumerate().for_each(|(i, &val)| {
        let byte_index = (i as isize / 32) as usize;
        let bit_index = ((i as isize & 31) * 2) as usize;
        match val {
            Some(true) => { bitboard[byte_index] |= 0b11 << bit_index; }
            Some(false) => { bitboard[byte_index] |= 0b10 << bit_index; }
            None => { () }
        } 
    });
}

// Need to take history into account, found some issue with double_three
pub fn get_ia(game: &mut game::Game) -> usize {
    let mut bitboard = [0_u64; 12];
    transpose_board_to_bitboard(game, &mut bitboard);

    // copy get_ia
    let mut rng = rand::thread_rng();

    match game.history.len() {
        0 => {
            println!("{}", "passé dans 0");
            180
        }
        2 => {
            println!("{}", "passé dans 1");
            match game.type_of_party {
                game::TypeOfParty::Pro => {
                    (180 + (capture::DIRS
                        .choose(&mut rng)
                        .expect("Error in random extraction")
                        * 3)) as usize
                }
                game::TypeOfParty::Longpro => {
                    (180 + (capture::DIRS
                        .choose(&mut rng)
                        .expect("Error in random extraction")
                        * 4)) as usize
                }
                game::TypeOfParty::Standard => {
                    (180 + (capture::DIRS
                        .choose(&mut rng)
                        .expect("Error in random extraction"))) as usize
                }
            }
        }
        _ => dumb_ia(game, &mut rng),
    }
}
