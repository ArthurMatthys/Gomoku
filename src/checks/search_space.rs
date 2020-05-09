use super::super::model::game;
use super::after_turn_check;
use super::capture;

fn around_area(game: &mut game::Game, index: isize, dir: isize) -> Vec<usize> {
    let mut ret = vec![];
    for i in [-1, 1].iter() {
        if capture::valid_dir(&index, i * dir, 1) {
            let new_index = index + dir * 1 * i;
            match game.board[new_index as usize] {
                None => ret.push(new_index as usize),
                _ => ()
            }
        }
    }
    ret
}

// 
pub fn search_space(game: &mut game::Game) -> Vec<usize> {
    let mut ret = vec![];
    for i in 0..361 {
        if game.board[i] == None {
            continue;
        } else {
            after_turn_check::DIRECTIONS.iter().for_each(|&x| {
                around_area(game, i as isize, x).iter().for_each(|&y| ret.push(y))
            });
        }
    }
    ret
}

