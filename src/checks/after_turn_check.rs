use array_tool::vec::*;

use super::super::model::game;
use super::capture;

pub const DIRECTIONS: [isize; 4] = [20, 19, 18, 1];

// Recursive function that recurses in the direction specified
// and counts the number of pawns of the same color
//fn explore(
//    board: &[Option<bool>; 361],
//    direction: isize,
//    index: &isize,
//    type_of_index: Option<bool>,
//    counter: usize,
//) -> isize {
//    match *index >= 0 && *index < 361 && board[*index as usize] == type_of_index {
//        true => match counter < 5 {
//            true => explore(
//                board,
//                direction,
//                &(*index + direction),
//                type_of_index,
//                counter + 1,
//            ),
//            false => counter as isize,
//        },
//        false => counter as isize,
//    }
//}

// Function that expands the if statement for more clarity
// It calls the explore function 2 times with the $direction and it's opposite
// And checks wether there are 5 or more pawn of the same color (on the same column)
fn check_explore(
    board: &[Option<bool>; 361],
    direction: isize,
    index_lpiece: &isize,
) -> Option<Vec<isize>> {
    let turn = board[*index_lpiece as usize];
    let mut indexes = vec![*index_lpiece];
    for i in [-1, 1].iter() {
        for j in 1..5 {
            if capture::valid_dir(index_lpiece, direction * i, j) {
                let new_index = i * j * direction + *index_lpiece;
                if board[new_index as usize] == turn {
                    indexes.push(new_index);
                } else {
                    break;
                }
            }
        }
    }
    match indexes.len() {
        0..=4 => None,
        5 => Some(indexes),
        6 => Some(indexes[1..5].to_vec()),
        7 => Some(indexes[2..5].to_vec()),
        8 => Some(indexes[3..5].to_vec()),
        9 => Some(vec![indexes[4]].to_vec()),
        _ => unreachable!(),
    }
    //    let nb_pawn_in_direction = explore(
    //        board,
    //        direction,
    //        index_lpiece,
    //        board[*index_lpiece as usize],
    //        0,
    //    );
    //    let nb_pawn_opposite_direction = explore(
    //        board,
    //        -direction,
    //        index_lpiece,
    //        board[*index_lpiece as usize],
    //        0,
    //    );
    //    // Check if the winning combination doesn't go overboard
    //    match capture::valid_dir(index_lpiece, direction, nb_pawn_in_direction - 1)
    //        && capture::valid_dir(index_lpiece, -direction, nb_pawn_opposite_direction - 1)
    //    {
    //        true => nb_pawn_in_direction + nb_pawn_opposite_direction - 1 >= 5,
    //        false => false,
    //    }
}

// Function that checks if a winner has been found
pub fn check_winner(game: &game::Game) -> Option<Vec<isize>> {
    let board = game.board;
    if let Some(index_lpiece) = game.history.last() {
        let call_mac = |&x| check_explore(&board, x, &(*index_lpiece as isize));
        let lst_indexes = DIRECTIONS
            .iter()
            .filter_map(call_mac)
            .collect::<Vec<Vec<isize>>>();
        match lst_indexes.len() {
            0 => None,
            1 => {
                let mut ret: Vec<isize> = vec![];
                lst_indexes[0].iter().for_each(|&x| ret.push(x));
                Some(ret)
            }
            2..=4 => {
                let mut ret: Vec<isize> = vec![];
                lst_indexes[0].iter().for_each(|&x| ret.push(x));
                for lst in lst_indexes[1..].iter() {
                    ret = ret.intersect(lst.to_vec());
                }
                Some(ret)
            }
            _ => unreachable!(),
        }
    } else {
        None
    }
}
