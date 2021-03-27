use array_tool::vec::*;

use super::super::model::game;
use super::super::render::board;
use super::capture;

pub const DIRECTIONS: [(isize, isize); 4] = [(1, 1), (1, 0), (1, -1), (0, 1)];

/// Function that expands the if statement for more clarity
/// It calls the explore function 2 times with the $direction and it's opposite
/// And checks wether there are 5 or more pawn of the same color (on the same column)
pub fn check_explore(
    board: &[[Option<bool>; board::SIZE_BOARD]; board::SIZE_BOARD],
    (dir_line, dir_col): (isize, isize),
    (line_lpiece, col_lpiece): &(isize, isize),
) -> Option<Vec<(isize, isize)>> {
    let turn = board[*line_lpiece as usize][*col_lpiece as usize];
    let mut indexes = vec![(*line_lpiece, *col_lpiece)];
    for i in [-1, 1].iter() {
        for j in 1..5 {
            if capture::valid_dir(&(*line_lpiece, *col_lpiece), (dir_line * i, dir_col * i), j) {
                let (new_index_line, new_index_col) = (
                    i * j * dir_line + *line_lpiece,
                    i * j * dir_col + *col_lpiece,
                );
                if board[new_index_line as usize][new_index_col as usize] == turn {
                    indexes.push((new_index_line, new_index_col));
                } else {
                    break;
                }
            } else {
                break;
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
}

// Function that checks if a winner has been found
pub fn check_winner(game: &game::Game) -> Option<Vec<(isize, isize)>> {
    let board = game.board;
    if let Some((lpiece_line, lpiece_col)) = game.history.last() {
        let call_mac =
            |&x| check_explore(&board, x, &(*lpiece_line as isize, *lpiece_col as isize));
        let lst_indexes = DIRECTIONS
            .iter()
            .filter_map(call_mac)
            .collect::<Vec<Vec<(isize, isize)>>>();
        match lst_indexes.len() {
            0 => None,
            1 => {
                let mut ret: Vec<(isize, isize)> = vec![];
                lst_indexes[0].iter().for_each(|&x| ret.push(x));
                Some(ret)
            }
            2..=4 => {
                let mut ret: Vec<(isize, isize)> = vec![];
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
