use super::super::model::game;
use super::super::render::board;

pub const DIRS: [isize; 8] = [20, 19, 18, 1, -1, -18, -19, -20];

pub fn valid_dir(index: &isize, dir: isize, moves: isize) -> bool {
    let final_index = *index + moves * dir;
    if final_index < 0 || final_index >= 361 {
        false
    } else {
        let delta_line =
            final_index / board::SIZE_BOARD as isize - *index / board::SIZE_BOARD as isize;
        let delta_col =
            final_index % board::SIZE_BOARD as isize - *index % board::SIZE_BOARD as isize;
        // println!("final_index:{} - delta_line:{} - delta_col:{} - index: {} - moves: {}", final_index, delta_line, delta_col, index, moves);
        // println!("-------------------");
        match dir {
            20 => delta_line == moves && delta_col == moves,
            19 => delta_line == moves && delta_col == 0,
            18 => delta_line == moves && delta_col == -moves,
            1 => delta_line == 0 && delta_col == moves,
            -1 => delta_line == 0 && delta_col == -moves,
            -18 => delta_line == -moves && delta_col == moves,
            -19 => delta_line == -moves && delta_col == 0,
            -20 => delta_line == -moves && delta_col == -moves,
            _ => unreachable!(),
        }
    }
}

// Recursive function that recurses in the direction specified
// and counts the number of pawns of the same color
fn explore_capture(
    board: &[Option<bool>; 361],
    direction: isize,
    index: &isize,
    type_of_index: bool,
    counter: usize,
) -> Option<(isize, isize)> {
    if *index >= 0 && *index < 361 && counter < 2 && board[*index as usize] == Some(!type_of_index)
    {
        explore_capture(
            board,
            direction,
            &(*index + direction),
            type_of_index,
            counter + 1,
        )
    } else if *index >= 0
        && *index < 361
        && counter == 2
        && board[*index as usize] == Some(type_of_index)
    {
        Some((index - direction, index - direction * 2))
    } else {
        None
    }
}

fn explore_capture_check(
    board: &[Option<bool>; 361],
    direction: isize,
    index: &isize,
    type_of_index: bool,
    counter: usize,
) -> Option<(isize, isize)> {
    if valid_dir(index, direction, 3) {
        explore_capture(
            board,
            direction,
            &(*index + direction),
            type_of_index,
            counter,
        )
    } else {
        None
    }
}

// Function that checks if a winner has been found
pub fn check_capture(game: &mut game::Game) -> Option<Vec<(isize, isize)>> {
    // let board = game.board;
    if let Some(index_lpiece) = game.history.last() {
        // I collect the type of the last piece addeded
        if let Some(piece) = game.board[*index_lpiece] {
            // Retrieves the map of true or false
            Some(
                DIRS.iter()
                    .filter_map(|&x| {
                        explore_capture_check(&game.board, x, &(*index_lpiece as isize), piece, 0)
                    })
                    .collect::<Vec<(isize, isize)>>(),
            )
        } else {
            None
        }
    } else {
        None
    }
}

pub fn find_capture(game: &mut game::Game) -> Vec<usize> {
    let mut ret: Vec<usize> = vec![];
    for i in 0..361 {
        if game.board[i] != None || game.is_forbidden_from_index(i) {
            continue;
        } else {
            game.change_board_value_hint(i);
            if let Some(taken) = check_capture(game) {
                if taken.len() > 0 {
                    ret.push(i);
                }
            }
            game.clear_last_move();
        }
    }
    ret
}

pub fn can_capture(game: &mut game::Game, to_capture: Vec<isize>) -> Option<Vec<usize>> {
    let mut ret: Vec<usize> = vec![];
    for i in 0..361 {
        if game.board[i] != None || game.is_forbidden_from_index(i) {
            continue;
        } else {
            game.change_board_value_hint(i);
            if let Some(taken) = check_capture(game) {
                for (a, b) in taken.iter() {
                    if to_capture.iter().any(|x| x == a || x == b) {
                        ret.push(i);
                    }
                }
            }
        }
        game.clear_last_move();
    }
    if ret.len() > 0 {
        Some(ret)
    } else {
        None
    }
}
