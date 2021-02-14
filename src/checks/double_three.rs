use super::super::model::board::Board;
use super::super::model::game;
use super::super::render::board::SIZE_BOARD;
use super::after_turn_check;
use super::capture;

// Datastruct used :
//      Index (0) : First way (-1 in for loop below) ==
//      Index (1) : Second way (1 in for loop below) ==
// [
//      [
//          nb_pions alliés avant espace ou pion ennemi,
//          si premier espace,
//          nb_pions alliés en 1er et 2ème espace,
//          2ème espace
//       ],
//      [0, 0, 0, 0]
//  ];
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
                        } else if parts[index_part][1] == 1 && parts[index_part][2] > 0 {
                            parts[index_part][2] = 0;
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

fn is_free_tree_hint(
    board: &mut Board,
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
            match board.get(new_index_line as usize, new_index_col as usize) {
                //  If I am on an empty position
                Some(None) => {
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
                // If I am on a competitors pawn, break
                Some(x) if x != current => {
                    if parts[index_part][1] == 1 && parts[index_part][2] == 0 {
                        parts[index_part][3] = 1;
                    } else if parts[index_part][1] == 1 && parts[index_part][2] > 0 {
                        parts[index_part][2] = 0;
                        parts[index_part][3] = 1;
                    }
                    break;
                }
                Some(_) => unreachable!(),
                None => break,
            }
            //            if valid_coord!(new_index_line, new_index_col) {
            //                match board[new_index_line as usize][new_index_col as usize] {
            //                    //  If I am on an empty position
            //                    None => {
            //                        // Check wether we already met an empty position
            //                        // If yes
            //                        if parts[index_part][1] == 1 {
            //                            if parts[index_part][2] + parts[index_part][0] != 0 {
            //                                parts[index_part][3] = 1;
            //                            }
            //                            break;
            //                        // Else, increment second index
            //                        } else {
            //                            parts[index_part][1] = 1;
            //                        }
            //                    }
            //                    // If I am on a competitors pawn, break
            //                    x if x != current => {
            //                        if parts[index_part][1] == 1 && parts[index_part][2] == 0 {
            //                            parts[index_part][3] = 1;
            //                        } else if parts[index_part][1] == 1 && parts[index_part][2] > 0 {
            //                            parts[index_part][2] = 0;
            //                            parts[index_part][3] = 1;
            //                        }
            //                        break;
            //                    }
            //                    // If I am on the player's pawn
            //                    x if x == current => {
            //                        // If I have met an empty position, increment the index 2 of the vec
            //                        if parts[index_part][1] == 1 {
            //                            parts[index_part][2] += 1;
            //                        // Else, increment the index on positon 0
            //                        } else {
            //                            parts[index_part][0] += 1;
            //                        }
            //                    }
            //                    _ => unreachable!(),
            //                };
            //            //                moves += 1;
            //            // Check next move
            //            // If we are on an invalid position, break
            //            } else {
            //                break;
            //            }
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
    board: &mut Board,
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

#[cfg(test)]
mod tests {
    // use super::super::handle_board::change_score_board_add;
    // use super::super::heuristic;
    use super::*;
    // use super::super::super::render::*;
    // use super::super::super::game::*;
    // use super::super::super::*;

    fn test_double_three_check_double_three_hint(
        white_pos: Vec<(usize, usize)>,
        black_pos: Vec<(usize, usize)>,
        actual_player: Option<bool>,
        (x, y): (isize, isize),
        expected_result: bool,
    ) -> bool {
        let mut test_board = [[None; SIZE_BOARD]; SIZE_BOARD];

        white_pos
            .iter()
            .for_each(|&(x, y)| test_board[x][y] = Some(true));
        black_pos
            .iter()
            .for_each(|&(x, y)| test_board[x][y] = Some(false));

        // Print initial configuration
        println!("// Initial configuration:");
        for i in 0..19 {
            print!("    // ");
            for j in 0..19 {
                match (i, j) {
                    z if z == (y, x) => print!("⊛"),
                    _ => match test_board[j as usize][i as usize] {
                        Some(true) => print!("⊖"),
                        Some(false) => print!("⊕"),
                        None => print!("_"),
                    },
                }
            }
            println!();
        }

        let ret = check_double_three_hint(&mut test_board.into(), actual_player, x, y);

        ret == expected_result
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _______⊕⊕⊛_________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_double_three_hint_0() {
        let black_pos = vec![(9, 8), (9, 7), (8, 6), (7, 6)];
        let white_pos = vec![];
        let pos2check = (9, 6);
        let expected_result = true;

        assert!(test_double_three_check_double_three_hint(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _______⊕⊕⊛_________
    // _________⊕_________
    // _________⊕_________
    // _________⊖_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_double_three_hint_1() {
        let black_pos = vec![(9, 8), (9, 7), (8, 6), (7, 6)];
        let white_pos = vec![(9, 9)];
        let pos2check = (9, 6);
        let expected_result = false;

        assert!(test_double_three_check_double_three_hint(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _______⊕⊕⊛_________
    // _________⊕_________
    // _________⊖_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_double_three_hint_2() {
        let black_pos = vec![(9, 5), (9, 7), (8, 6), (7, 6)];
        let white_pos = vec![(9, 8)];
        let pos2check = (9, 6);
        let expected_result = false;

        assert!(test_double_three_check_double_three_hint(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _______⊕⊕⊛_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_double_three_hint_3() {
        let black_pos = vec![(9, 5), (9, 7), (8, 6), (7, 6)];
        let white_pos = vec![];
        let pos2check = (9, 6);
        let expected_result = true;

        assert!(test_double_three_check_double_three_hint(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // ___________________
    // _______⊕⊕⊛_________
    // _________⊕_________
    // _________⊖_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_double_three_hint_4() {
        let black_pos = vec![(9, 4), (9, 7), (8, 6), (7, 6)];
        let white_pos = vec![(9, 8)];
        let pos2check = (9, 6);
        let expected_result = false;

        assert!(test_double_three_check_double_three_hint(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // ___________________
    // _______⊕⊕⊛_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_double_three_hint_5() {
        let black_pos = vec![(9, 4), (9, 7), (8, 6), (7, 6)];
        let white_pos = vec![];
        let pos2check = (9, 6);
        let expected_result = true;

        assert!(test_double_three_check_double_three_hint(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // ___________________
    // ______⊕⊕_⊛_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_double_three_hint_6() {
        let black_pos = vec![(9, 4), (9, 7), (6, 6), (7, 6)];
        let white_pos = vec![];
        let pos2check = (9, 6);
        let expected_result = true;

        assert!(test_double_three_check_double_three_hint(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // ___________________
    // ______⊕⊕_⊛_________
    // _________⊕_________
    // _________⊖_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_double_three_hint_7() {
        let black_pos = vec![(9, 4), (9, 7), (6, 6), (7, 6)];
        let white_pos = vec![(9, 8)];
        let pos2check = (9, 6);
        let expected_result = false;

        assert!(test_double_three_check_double_three_hint(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // ______⊕⊕_⊛_________
    // ___________________
    // _________⊕_________
    // _________⊖_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_double_three_hint_8() {
        let black_pos = vec![(9, 8), (6, 6), (7, 6), (9, 5)];
        let white_pos = vec![(9, 9)];
        let pos2check = (9, 6);
        let expected_result = false;

        assert!(test_double_three_check_double_three_hint(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // _________⊖_________
    // _________⊕_________
    // ___________________
    // ______⊕⊕_⊛_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_double_three_hint_9() {
        let black_pos = vec![(9, 4), (9, 7), (6, 6), (7, 6)];
        let white_pos = vec![(9, 3)];
        let pos2check = (9, 6);
        let expected_result = false;

        assert!(test_double_three_check_double_three_hint(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // ___________________
    // ______⊕⊕_⊛⊖________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_double_three_hint_10() {
        let black_pos = vec![(9, 4), (9, 7), (6, 6), (7, 6)];
        let white_pos = vec![(10, 6)];
        let pos2check = (9, 6);
        let expected_result = false;

        assert!(test_double_three_check_double_three_hint(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // ___________________
    // ______⊕⊕⊖⊛_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_double_three_hint_11() {
        let black_pos = vec![(9, 4), (9, 7), (6, 6), (7, 6)];
        let white_pos = vec![(8, 6)];
        let pos2check = (9, 6);
        let expected_result = false;

        assert!(test_double_three_check_double_three_hint(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊖_________
    // ______⊕⊕_⊛_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_double_three_hint_12() {
        let black_pos = vec![(9, 4), (9, 7), (6, 6), (7, 6)];
        let white_pos = vec![(9, 5)];
        let pos2check = (9, 6);
        let expected_result = false;

        assert!(test_double_three_check_double_three_hint(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_⊕_______
    // _________⊖⊕________
    // ______⊕⊕_⊛_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_double_three_hint_13() {
        let black_pos = vec![(9, 4), (9, 7), (6, 6), (7, 6), (10, 5), (11, 4)];
        let white_pos = vec![(9, 5)];
        let pos2check = (9, 6);
        let expected_result = true;

        assert!(test_double_three_check_double_three_hint(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ____________⊖______
    // _________⊕_⊕_______
    // _________⊖⊕________
    // ______⊕⊕_⊛_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_double_three_hint_14() {
        let black_pos = vec![(9, 4), (9, 7), (6, 6), (7, 6), (10, 5), (11, 4)];
        let white_pos = vec![(9, 5), (12, 3)];
        let pos2check = (9, 6);
        let expected_result = false;

        assert!(test_double_three_check_double_three_hint(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // --------------------------------------------------------------------------
    // Check Global Double_tree function (not the hint one)
    // We duplicate it here because the game datastruct is a pain to create for tests

    fn is_free_tree(
        board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
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
                        Some(x) if x != current => {
                            if parts[index_part][1] == 1 && parts[index_part][2] == 0 {
                                parts[index_part][3] = 1;
                            } else if parts[index_part][1] == 1 && parts[index_part][2] > 0 {
                                parts[index_part][2] = 0;
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

    fn check_double_three(
        board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
        actual_player: Option<bool>,
    ) -> Vec<(usize, usize)> {
        let mut ret = vec![];
        let pawn_current_player = actual_player.unwrap();
        for i in 0..19 {
            for j in 0..19 {
                let mut nbr_free_tree = 0;
                if board[i][j] != None {
                    continue;
                } else {
                    after_turn_check::DIRECTIONS.iter().for_each(|&x| {
                        if is_free_tree(board, (i as isize, j as isize), pawn_current_player, x) {
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

    fn test_double_three_check_double_three(
        white_pos: Vec<(usize, usize)>,
        black_pos: Vec<(usize, usize)>,
        actual_player: Option<bool>,
        (x, y): (isize, isize),
        expected_result: Vec<(usize, usize)>,
    ) -> bool {
        let mut test_board = [[None; SIZE_BOARD]; SIZE_BOARD];

        white_pos
            .iter()
            .for_each(|&(x, y)| test_board[x][y] = Some(true));
        black_pos
            .iter()
            .for_each(|&(x, y)| test_board[x][y] = Some(false));

        // Print initial configuration
        println!("// Initial configuration:");
        for i in 0..19 {
            print!("    // ");
            for j in 0..19 {
                match (i, j) {
                    z if z == (y, x) => print!("⊛"),
                    _ => match test_board[j as usize][i as usize] {
                        Some(true) => print!("⊖"),
                        Some(false) => print!("⊕"),
                        None => print!("_"),
                    },
                }
            }
            println!();
        }

        let ret = check_double_three(&mut test_board, actual_player);

        println!("DEBUG-XXX");
        print!("[");
        ret.iter().enumerate().for_each(|(i, (x, y))| {
            if i < ret.len() - 1 {
                print!("({}:{}),", x, y)
            } else {
                print!("({}:{})", x, y)
            }
        });
        println!("]");

        ret == expected_result
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _______⊕⊕⊛_________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_global_double_three_0() {
        let black_pos = vec![(9, 8), (9, 7), (8, 6), (7, 6)];
        let white_pos = vec![];
        let pos2check = (9, 6);
        let expected_result = vec![(9, 6)];

        assert!(test_double_three_check_double_three(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _______⊕⊕⊛_________
    // _________⊕_________
    // _________⊕_________
    // _________⊖_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_global_double_three_1() {
        let black_pos = vec![(9, 8), (9, 7), (8, 6), (7, 6)];
        let white_pos = vec![(9, 9)];
        let pos2check = (9, 6);
        let expected_result = vec![];

        assert!(test_double_three_check_double_three(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _______⊕⊕⊛_________
    // _________⊕_________
    // _________⊖_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_global_double_three_2() {
        let black_pos = vec![(9, 5), (9, 7), (8, 6), (7, 6)];
        let white_pos = vec![(9, 8)];
        let pos2check = (9, 6);
        let expected_result = vec![];

        assert!(test_double_three_check_double_three(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _______⊕⊕⊛_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_global_double_three_3() {
        let black_pos = vec![(9, 5), (9, 7), (8, 6), (7, 6)];
        let white_pos = vec![];
        let pos2check = (9, 6);
        let expected_result = vec![(9, 6)];

        assert!(test_double_three_check_double_three(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // ___________________
    // _______⊕⊕⊛_________
    // _________⊕_________
    // _________⊖_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_global_double_three_4() {
        let black_pos = vec![(9, 4), (9, 7), (8, 6), (7, 6)];
        let white_pos = vec![(9, 8)];
        let pos2check = (9, 6);
        let expected_result = vec![];

        assert!(test_double_three_check_double_three(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // ___________________
    // _______⊕⊕⊛_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_global_double_three_5() {
        let black_pos = vec![(9, 4), (9, 7), (8, 6), (7, 6)];
        let white_pos = vec![];
        let pos2check = (9, 6);
        let expected_result = vec![(9, 6)];

        assert!(test_double_three_check_double_three(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // ___________________
    // ______⊕⊕_⊛_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_global_double_three_6() {
        let black_pos = vec![(9, 4), (9, 7), (6, 6), (7, 6)];
        let white_pos = vec![];
        let pos2check = (9, 6);
        let expected_result = vec![(9, 6)];

        assert!(test_double_three_check_double_three(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // ___________________
    // ______⊕⊕_⊛_________
    // _________⊕_________
    // _________⊖_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_global_double_three_7() {
        let black_pos = vec![(9, 4), (9, 7), (6, 6), (7, 6)];
        let white_pos = vec![(9, 8)];
        let pos2check = (9, 6);
        let expected_result = vec![];

        assert!(test_double_three_check_double_three(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // ______⊕⊕_⊛_________
    // ___________________
    // _________⊕_________
    // _________⊖_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_global_double_three_8() {
        let black_pos = vec![(9, 8), (6, 6), (7, 6), (9, 5)];
        let white_pos = vec![(9, 9)];
        let pos2check = (9, 6);
        let expected_result = vec![];

        assert!(test_double_three_check_double_three(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // _________⊖_________
    // _________⊕_________
    // ___________________
    // ______⊕⊕_⊛_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_global_double_three_9() {
        let black_pos = vec![(9, 4), (9, 7), (6, 6), (7, 6)];
        let white_pos = vec![(9, 3)];
        let pos2check = (9, 6);
        let expected_result = vec![];

        assert!(test_double_three_check_double_three(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // ___________________
    // ______⊕⊕_⊛⊖________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_global_double_three_10() {
        let black_pos = vec![(9, 4), (9, 7), (6, 6), (7, 6)];
        let white_pos = vec![(10, 6)];
        let pos2check = (9, 6);
        let expected_result = vec![];

        assert!(test_double_three_check_double_three(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // ___________________
    // ______⊕⊕⊖⊛_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_global_double_three_11() {
        let black_pos = vec![(9, 4), (9, 7), (6, 6), (7, 6)];
        let white_pos = vec![(8, 6)];
        let pos2check = (9, 6);
        let expected_result = vec![];

        assert!(test_double_three_check_double_three(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊖_________
    // ______⊕⊕_⊛_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_global_double_three_12() {
        let black_pos = vec![(9, 4), (9, 7), (6, 6), (7, 6)];
        let white_pos = vec![(9, 5)];
        let pos2check = (9, 6);
        let expected_result = vec![];

        assert!(test_double_three_check_double_three(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_⊕_______
    // _________⊖⊕________
    // ______⊕⊕_⊛_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_global_double_three_13() {
        let black_pos = vec![(9, 4), (9, 7), (6, 6), (7, 6), (10, 5), (11, 4)];
        let white_pos = vec![(9, 5)];
        let pos2check = (9, 6);
        let expected_result = vec![(9, 6)];

        assert!(test_double_three_check_double_three(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ____________⊖______
    // _________⊕_⊕_______
    // _________⊖⊕________
    // ______⊕⊕_⊛_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn double_threat_find_global_double_three_14() {
        let black_pos = vec![(9, 4), (9, 7), (6, 6), (7, 6), (10, 5), (11, 4)];
        let white_pos = vec![(9, 5), (12, 3)];
        let pos2check = (9, 6);
        let expected_result = vec![];

        assert!(test_double_three_check_double_three(
            white_pos,
            black_pos,
            Some(false),
            pos2check,
            expected_result
        ))
    }
}
