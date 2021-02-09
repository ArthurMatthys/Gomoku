use super::super::ia::heuristic;
use super::super::model::game;
use super::super::render::board;

pub const DIRS: [(isize, isize); 8] = [
    (1, 1),
    (1, 0),
    (1, -1),
    (0, 1),
    (0, -1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];

pub const DIRS_0: [(isize, isize); 9] = [
    (1, 1),
    (1, 0),
    (1, -1),
    (0, 1),
    (0, 0),
    (0, -1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];

pub fn valid_dir(
    (line, col): &(isize, isize),
    (dir_line, dir_col): (isize, isize),
    moves: isize,
) -> bool {
    let (final_line, final_col) = (*line + moves * dir_line, *col + moves * dir_col);
    if final_line < 0 || final_col < 0 || final_line >= 19 || final_col >= 19 {
        false
    } else {
        let delta_line = moves * dir_line;
        // final_index / board::SIZE_BOARD as isize - *index / board::SIZE_BOARD as isize;
        let delta_col = moves * dir_col;
        // final_index % board::SIZE_BOARD as isize - *index % board::SIZE_BOARD as isize;
        // println!("final_index:{} - delta_line:{} - delta_col:{} - index: {} - moves: {}", final_index, delta_line, delta_col, index, moves);
        // println!("-------------------");
        match (dir_line, dir_col) {
            (1, 1) => delta_line == moves && delta_col == moves,
            (1, 0) => delta_line == moves && delta_col == 0,
            (1, -1) => delta_line == moves && delta_col == -moves,
            (0, 1) => delta_line == 0 && delta_col == moves,
            (0, -1) => delta_line == 0 && delta_col == -moves,
            (-1, 1) => delta_line == -moves && delta_col == moves,
            (-1, 0) => delta_line == -moves && delta_col == 0,
            (-1, -1) => delta_line == -moves && delta_col == -moves,
            _ => unreachable!(),
        }
    }
}

// Recursive function that recurses in the direction specified
// and counts the number of pawns of the same color
fn explore_capture(
    board: &[[Option<bool>; board::SIZE_BOARD]; board::SIZE_BOARD],
    (direction_line, direction_col): (isize, isize),
    (index_line, index_col): &(isize, isize),
    type_of_index: bool,
    counter: usize,
) -> Option<((isize, isize), (isize, isize))> {
    if *index_line >= 0
        && *index_line < 361
        && *index_col >= 0
        && *index_col < 361
        && counter < 2
        && board[*index_line as usize][*index_col as usize] == Some(!type_of_index)
    {
        explore_capture(
            board,
            (direction_line, direction_col),
            &(*index_line + direction_line, *index_col + direction_col),
            type_of_index,
            counter + 1,
        )
    } else if *index_line >= 0
        && *index_col >= 0
        && *index_line < 361
        && *index_col < 361
        && counter == 2
        && board[*index_line as usize][*index_col as usize] == Some(type_of_index)
    {
        Some((
            (*index_line - direction_line, *index_col - direction_col),
            (
                *index_line - direction_line * 2,
                *index_col - direction_col * 2,
            ),
        ))
    } else {
        None
    }
}

fn explore_capture_check(
    board: &[[Option<bool>; board::SIZE_BOARD]; board::SIZE_BOARD],
    (direction_line, direction_col): (isize, isize),
    (index_line, index_col): &(isize, isize),
    type_of_index: bool,
    counter: usize,
) -> Option<((isize, isize), (isize, isize))> {
    if valid_dir(
        &(*index_line, *index_col),
        (direction_line, direction_col),
        3,
    ) {
        explore_capture(
            board,
            (direction_line, direction_col),
            &(*index_line + direction_line, *index_col + direction_col),
            type_of_index,
            counter,
        )
    } else {
        None
    }
}

// Function that checks if a winner has been found
pub fn check_capture(game: &mut game::Game) -> Option<Vec<((isize, isize), (isize, isize))>> {
    // let board = game.board;
    if let Some((line, col)) = game.history.last() {
        // I collect the type of the last piece addeded
        if let Some(piece) = game.board[*line][*col] {
            // Retrieves the map of true or false
            Some(
                DIRS.iter()
                    .filter_map(|&x| {
                        explore_capture_check(
                            &game.board,
                            x,
                            &(*line as isize, *col as isize),
                            piece,
                            0,
                        )
                    })
                    .collect::<Vec<((isize, isize), (isize, isize))>>(),
            )
        } else {
            None
        }
    } else {
        None
    }
}

pub fn find_capture(game: &mut game::Game) -> Vec<(usize, usize)> {
    let mut ret: Vec<(usize, usize)> = vec![];
    for i in 0..19 {
        for j in 0..19 {
            if game.board[i][j] != None || game.is_forbidden_from_index(i, j) {
                continue;
            } else {
                game.change_board_value_hint(i, j);
                if let Some(taken) = check_capture(game) {
                    if taken.len() > 0 {
                        ret.push((i, j));
                    }
                }
                game.clear_last_move();
            }
        }
    }
    ret
}

pub fn can_capture(game: &mut game::Game, to_capture: Vec<(isize, isize)>) -> bool {
    let score_board = heuristic::evaluate_board(&mut game.board.into());
    for &(x, y) in to_capture.iter() {
        let new_x = x as usize;
        let new_y = y as usize;
        for dir in 0..4 {
            match score_board.get(new_x, new_y, dir) {
                (a, l, r) if a == 2 && ((l == Some(false) && r == Some(true)) || (l == Some (true) && r == Some(false))) => {
                    return true
                }
                _ => (),
            }
        }
    }
    false
    //    for i in 0..19 {
    //        for j in 0..19 {
    //            if game.board[i][j] != None || game.is_forbidden_from_index(i, j) {
    //                continue;
    //            } else {
    //                game.change_board_value_hint(i, j);
    //                if let Some(taken) = check_capture(game) {
    //                    for ((a, b), (c, d)) in taken.iter() {
    //                        // (a,b)
    //                        if to_capture
    //                            .iter()
    //                            .any(|(x, y)| x == a && y == b || x == c && y == d)
    //                        {
    //                            ret.push((i, j));
    //                        }
    //                    }
    //                }
    //            }
    //            game.clear_last_move();
    //        }
    //    }
    //    if ret.len() > 0 {
    //        Some(ret)
    //    } else {
    //        None
    //    }
}
