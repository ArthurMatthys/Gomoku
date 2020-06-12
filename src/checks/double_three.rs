use super::super::model::game;
use super::super::render::board::SIZE_BOARD;
use super::after_turn_check;
use super::capture;

fn is_free_tree(
    game: &mut game::Game,
    (line, col): (isize, isize),
    current: bool,
    (dir_line, dir_col): (isize, isize),
) -> bool {
    let mut parts = [[0, 0, 0, 0], [0, 0, 0, 0]];
    for i in [-1, 1].iter() {
        let index_part: usize = ((i + 1) / 2) as usize;
        let mut moves: isize = 1;
        loop {
            if capture::valid_dir(&(line, col), (i * dir_line, i * dir_col), moves) {
                let (new_index_line, new_index_col) =
                    (line + dir_line * moves * i, col + dir_col * moves * i);
                match game.board[new_index_line as usize][new_index_col as usize] {
                    //  If I am on an empty position
                    None => {
                        // Check wether we already met an empty position
                        // If yes
                        if parts[index_part][1] == 1 {
                            if parts[index_part][2] + parts[index_part][0] != 0 {
                                parts[index_part][3] = 1;
                            }
                            break;
                        // Else, increment second index
                        } else {
                            parts[index_part][1] = 1;
                        }
                    }
                    // If I am on a competitors pawn, break
                    Some(x) if x != current => {
                        if parts[index_part][1] == 1 && parts[index_part][2] == 0 {
                            parts[index_part][3] = 1;
                        }
                        break;
                    }
                    // If I am on the player's pawn
                    Some(x) if x == current => {
                        // If I have met an empty position, increment the index 2 of the vec
                        if parts[index_part][1] == 1 {
                            parts[index_part][2] += 1;
                        // Else, increment the index on positon 0
                        } else {
                            parts[index_part][0] += 1;
                        }
                    }
                    _ => unreachable!(),
                };
                // Check next move
                moves += 1;
            // If we are on an invalid position, break
            } else {
                break;
            }
        }
    }
    let tot = (
        parts[0][0] + parts[1][0],
        parts[0][1] + parts[1][1],
        parts[0][2] + parts[1][2],
        parts[0][3] + parts[1][3],
    );
    if tot.3 >= 1
        && tot.1 == 2
        && (tot.0 + tot.2 == 2 && ((parts[0][2] * parts[1][2] == 0) || tot.0 != 0))
    {
        true
    } else {
        false
    }
}

pub fn check_double_three(game: &mut game::Game) -> Vec<(usize, usize)> {
    let mut ret = vec![];
    let pawn_current_player = !game
        .player_to_pawn()
        .expect("Could not retrieve player pawn");
    for i in 0..19 {
        for j in 0..19 {
            let mut nbr_free_tree = 0;
            if game.board[i][j] != None {
                continue;
            } else {
                after_turn_check::DIRECTIONS.iter().for_each(|&x| {
                    if is_free_tree(game, (i as isize, j as isize), pawn_current_player, x) {
                        nbr_free_tree += 1;
                    }
                });
            }
            if nbr_free_tree >= 2 {
                ret.push((i, j));
            }
        }
    }
    ret
}

macro_rules! valid_coord {
    ($e:expr, $v:expr) => {
        $e >= 0 && $v >= 0 && ($e as usize) < SIZE_BOARD && ($v as usize) < SIZE_BOARD
    };
}

fn is_free_tree_hint(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    (line, col): (isize, isize),
    current: Option<bool>,
    (dir_line, dir_col): (isize, isize),
) -> bool {
    let mut parts = [[0, 0, 0, 0], [0, 0, 0, 0]];
    for i in [-1, 1].iter() {
        let index_part: usize = ((i + 1) / 2) as usize;
        //        let mut moves: isize = 1;
        let new_dir_line = i * dir_line;
        let new_dir_col = i * dir_col;
        let mut new_index_line = line;
        let mut new_index_col = col;
        loop {
            //            if capture::valid_dir(&(line, col), (i * dir_line, i * dir_col), moves) {
            //                let (new_index_line, new_index_col) =
            //                    (line + dir_line * moves * i, col + dir_col * moves * i);
            new_index_line += new_dir_line;
            new_index_col += new_dir_col;
            if valid_coord!(new_index_line, new_index_col) {
                match board[new_index_line as usize][new_index_col as usize] {
                    //  If I am on an empty position
                    None => {
                        // Check wether we already met an empty position
                        // If yes
                        if parts[index_part][1] == 1 {
                            if parts[index_part][2] + parts[index_part][0] != 0 {
                                parts[index_part][3] = 1;
                            }
                            break;
                        // Else, increment second index
                        } else {
                            parts[index_part][1] = 1;
                        }
                    }
                    // If I am on a competitors pawn, break
                    x if x != current => {
                        if parts[index_part][1] == 1 && parts[index_part][2] == 0 {
                            parts[index_part][3] = 1;
                        }
                        break;
                    }
                    // If I am on the player's pawn
                    x if x == current => {
                        // If I have met an empty position, increment the index 2 of the vec
                        if parts[index_part][1] == 1 {
                            parts[index_part][2] += 1;
                        // Else, increment the index on positon 0
                        } else {
                            parts[index_part][0] += 1;
                        }
                    }
                    _ => unreachable!(),
                };
            //                moves += 1;
            // Check next move
            // If we are on an invalid position, break
            } else {
                break;
            }
        }
    }
    let tot = (
        parts[0][0] + parts[1][0],
        parts[0][1] + parts[1][1],
        parts[0][2] + parts[1][2],
        parts[0][3] + parts[1][3],
    );
    if tot.3 >= 1
        && tot.1 == 2
        && (tot.0 + tot.2 == 2 && ((parts[0][2] * parts[1][2] == 0) || tot.0 != 0))
    {
        true
    } else {
        false
    }
}

pub fn check_double_three_hint(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    actual_player: Option<bool>,
    x: isize,
    y: isize,
) -> bool {
    if x < 1 || x >= (SIZE_BOARD as isize - 1) || y < 1 || y >= (SIZE_BOARD as isize - 1) {
        return false;
    }
    let mut nbr_free_tree = 0;
    after_turn_check::DIRECTIONS.iter().for_each(|&dir| {
        if nbr_free_tree >= 2 || is_free_tree_hint(board, (x, y), actual_player, dir) {
            nbr_free_tree += 1;
        }
    });
    nbr_free_tree >= 2
}
