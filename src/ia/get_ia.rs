// use super::super::model::point;
// use rand::distributions::{Distribution, Uniform};
// use rand::thread_rng;
use super::super::checks::capture;
use super::super::model::game;
use super::super::render::board::SIZE_BOARD;
use super::handle_board::{
    board_state_win, change_board, change_board_hint, find_continuous_threats, get_space,
    null_move_heuristic, remove_last_pawn, remove_last_pawn_hint,
};
// use super::super::model::player;
use super::heuristic;
use super::zobrist;
use rand::seq::SliceRandom;
use std::time;
// use super::super::player;

const MIN_INFINITY: i64 = i64::min_value() + 1;
const MAX_INFINITY: i64 = i64::max_value();
const DEPTH_THREATS: i8 = 10;

macro_rules! get_opp {
    ($e:expr) => {
        match $e {
            Some(a) => Some(!a),
            _ => unreachable!(),
        }
    };
}

fn find_winning_align(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    actual: Option<bool>,
) -> bool {
    for line in 0..SIZE_BOARD {
        for col in 0..SIZE_BOARD {
            if board[line][col] == actual {
                for dir in 0..4 {
                    if score_board[line][col][dir].0 >= 5 {
                        return true;
                    }
                }
            }
        }
    }
    false
}

// negamax_try
fn ab_negamax(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    table: &[[[u64; 2]; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    zhash: &mut u64,
    tt: &mut Vec<zobrist::TT>,
    current_depth: &mut i8,
    actual: Option<bool>,
    actual_catch: &mut isize,
    opp_catch: &mut isize,
    alpha: &mut i64,
    beta: &mut i64,
    color: &mut i8,
    depth_max: &i8,
) -> (i64, Option<(usize, usize)>) {
    let mut tte = zobrist::retrieve_tt_from_hash(tt, zhash);
    let alpha_orig = *alpha;

    if tte.is_valid && tte.depth == *depth_max - *current_depth {
        if tte.r#type == zobrist::TypeOfEl::Exact {
            return (tte.value, tte.r#move);
        } else if tte.r#type == zobrist::TypeOfEl::Lowerbound {
            *alpha = i64::max(*alpha, tte.value);
        } else if tte.r#type == zobrist::TypeOfEl::Upperbound {
            *beta = i64::min(*beta, tte.value);
        }

        if *alpha >= *beta {
            return (tte.value, tte.r#move);
        }
    }

    if *opp_catch >= 5 {
        return (-heuristic::INSTANT_WIN * (*current_depth as i64 + 1), None);
    } else if find_winning_align(board, score_board, actual) {
        return (heuristic::INSTANT_WIN * (*current_depth as i64 + 1), None);
    }
    if *current_depth == *depth_max {
        let weight = heuristic::first_heuristic_hint(
            board,
            score_board,
            actual,
            actual_catch,
            opp_catch,
            &mut (*depth_max - *current_depth),
        );
        return (weight, None);
    }

    // Otherwise bubble up values from below
    let mut best_move: Option<(usize, usize)> = None;
    let mut best_score = MIN_INFINITY;
    let mut trig = false;

    if tte.is_valid && tte.depth >= *depth_max - *current_depth {
        // println!("rentré");
        if let Some((line, col)) = tte.r#move {
            if board[line][col] == None {
                let removed = change_board(board, score_board, line, col, actual, table, zhash);
                *actual_catch += removed.len() as isize;

                // Recurse
                let (recursed_score, _) = ab_negamax(
                    board,
                    table,
                    score_board,
                    zhash,
                    tt,
                    &mut (*current_depth + 1),
                    get_opp!(actual),
                    opp_catch,
                    actual_catch,
                    &mut (-*beta),
                    &mut (-*alpha),
                    &mut (-*color),
                    depth_max,
                );
                let x = -recursed_score;

                *actual_catch -= removed.len() as isize;
                remove_last_pawn(board, score_board, line, col, actual, removed, table, zhash);

                if x >= *beta {
                    // println!("rentré_cutoff");
                    best_score = x;
                    best_move = tte.r#move;
                    trig = true;
                }
                // else {
                //     best_score = MIN_INFINITY;
                //     best_move = None;
                // }
            }
        }
    }

    if !trig {
        // Collect moves
        let available_positions = get_space(board, score_board, actual, *actual_catch);

        for (line, col, _) in available_positions {
            let removed = change_board(board, score_board, line, col, actual, table, zhash);
            *actual_catch += removed.len() as isize;

            // Recurse
            let (recursed_score, _) = ab_negamax(
                board,
                table,
                score_board,
                zhash,
                tt,
                &mut (*current_depth + 1),
                get_opp!(actual),
                opp_catch,
                actual_catch,
                &mut (-*beta),
                &mut (-*alpha),
                &mut (-*color),
                depth_max,
            );

            let x = -recursed_score;
            if x > best_score {
                best_score = x;
                best_move = Some((line, col));
            }
            if x > *alpha {
                *alpha = x;
                best_move = Some((line, col));
            }

            *actual_catch -= removed.len() as isize;
            remove_last_pawn(board, score_board, line, col, actual, removed, table, zhash);

            if *alpha >= *beta {
                best_score = *alpha;
                best_move = Some((line, col));
                break;
                // return (*alpha, best_move);
            }
        }
    }

    if best_score <= alpha_orig {
        tte.r#type = zobrist::TypeOfEl::Upperbound;
    } else if best_score >= *beta {
        tte.r#type = zobrist::TypeOfEl::Lowerbound;
    } else {
        tte.r#type = zobrist::TypeOfEl::Exact;
    }
    tte.is_valid = true;
    tte.key = *zhash;
    tte.value = best_score;
    tte.r#move = best_move;
    tte.depth = *depth_max - *current_depth;
    zobrist::store_tt_entry(tt, zhash, tte);

    (best_score, best_move)
}

fn mtdf(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    table: &[[[u64; 2]; SIZE_BOARD]; SIZE_BOARD],
    zhash: &mut u64,
    tt: &mut Vec<zobrist::TT>,
    actual: Option<bool>,
    actual_catch: &mut isize,
    opp_catch: &mut isize,
    beta: &mut i64,
    depth_max: &i8,
    firstguess: i64,
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
) -> (i64, (usize, usize)) {
    let mut g = firstguess;
    let mut ret = (0, (0, 0));
    let mut upperbnd = MAX_INFINITY;
    let mut lowerbnd = MIN_INFINITY;

    while lowerbnd < upperbnd {
        // let mut score_board2 = heuristic::evaluate_board(board);
        let mut actual_catch2 = *actual_catch;
        let mut opp_catch2 = *opp_catch;
        if g == lowerbnd {
            *beta = g + 1;
        } else {
            *beta = g;
        }
        // *beta = i64::max(g, lowerbnd + 1);
        let (score, r#move): (i64, Option<(usize, usize)>) = ab_negamax(
            board,
            table,
            score_board,
            zhash,
            tt,
            &mut 0,
            actual,
            &mut actual_catch2,
            &mut opp_catch2,
            &mut (*beta - 1),
            beta,
            &mut 1,
            depth_max,
        );
        ret = (score, r#move.unwrap());
        g = score;
        if g < *beta {
            upperbnd = g;
        } else {
            lowerbnd = g;
        }
    }
    ret
}

fn iterative_deepening_mtdf(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    table: &[[[u64; 2]; SIZE_BOARD]; SIZE_BOARD],
    zhash: &mut u64,
    tt: &mut Vec<zobrist::TT>,
    actual: Option<bool>,
    actual_catch: &mut isize,
    opp_catch: &mut isize,
    beta: &mut i64,
    depth_max: &i8,
    game: &mut game::Game,
    start_time: &time::Instant
) -> (usize, usize) {
    let mut ret = (0, 0);
    let mut f = match actual {
        Some(true) => game.firstguess.0,
        Some(false) => game.firstguess.1,
        None => unreachable!(),
    };
    let limit_duration = time::Duration::from_millis(480);

    // println!("before- f: {}", f);
    // for d in [2, 3, 5].iter() {
    for d in (2..(depth_max + 1)).step_by(2) {
        let mut beta2 = *beta;
        let mut actual_catch2 = *actual_catch;
        let mut opp_catch2 = *opp_catch;

        // for d in (1..DEPTH_MAX).step_by(2) {
        //    println!("debug: {}|{}|{}|{}|{}|{}|", *alpha, *beta, actual_catch2, opp_catch2, d, *zhash);
        let end = time::Instant::now();
        if end.duration_since(*start_time) >= limit_duration {
            break;
        }
        let (score, r#move) = mtdf(
            board,
            table,
            zhash,
            tt,
            actual,
            &mut actual_catch2,
            &mut opp_catch2,
            &mut beta2,
            &d,
            f,
            score_board,
        );
        ret = r#move;
        f = score;
        // println!("debug2: {}|{}|{}|{}|{}|{}|", *alpha, *beta, actual_catch2, opp_catch2, d, *zhash);
    }
    match actual {
        Some(true) => game.firstguess.0 = f,
        Some(false) => game.firstguess.1 = f,
        None => unreachable!(),
    };
    // println!("after- f: {}", f);
    ret
}

fn ia(
    game: &mut game::Game,
    (table, mut hash): (&[[[u64; 2]; SIZE_BOARD]; SIZE_BOARD], u64),
    depth_max: &i8,
    start_time: &time::Instant
) -> (usize, usize) {
    let mut player_catch = game.get_actual_player().nb_of_catch;
    let mut opponent_catch = game.get_opponent().nb_of_catch;
    let mut board = game.board;
    let mut score_board = heuristic::evaluate_board(&mut board);
    let pawn = game.player_to_pawn();
    let mut tt = zobrist::initialize_transposition_table();

    if let Some((x, y)) = null_move_heuristic(
        &mut board,
        &mut score_board,
        pawn,
        &mut opponent_catch,
        &mut player_catch,
    ) {
        println!("Answer null move ({},{})", x, y);
        return (x, y);
    }
    if let Some((x, y)) = find_continuous_threats(
        &mut board,
        &mut score_board,
        pawn,
        &mut player_catch,
        &mut opponent_catch,
        &mut DEPTH_THREATS,
        &mut 0,
        true,
    ) {
        println!("find threat ({},{})", x, y);
        return (x, y);
    }
    iterative_deepening_mtdf(
        &mut board,
        &mut score_board,
        table,
        &mut hash,
        &mut tt,
        pawn,
        &mut player_catch,
        &mut opponent_catch,
        &mut MAX_INFINITY,
        depth_max,
        game,
        start_time
    )
}

pub fn get_ia(
    game: &mut game::Game,
    ztable: &[[[u64; 2]; SIZE_BOARD]; SIZE_BOARD],
    depth_max: &i8,
    start_time: &time::Instant
) -> (usize, usize) {
    let hash: u64 = zobrist::board_to_zhash(&game.board, ztable);
    let mut rng = rand::thread_rng();

    match game.history.len() {
        0 => (9, 9),
        2 => {
            let (dir_line, dir_col) = capture::DIRS
                .choose(&mut rng)
                .expect("Error in random extraction");
            match game.type_of_party {
                game::TypeOfParty::Pro => ((9 + dir_line * 3) as usize, (9 + dir_col * 3) as usize),
                game::TypeOfParty::Longpro => {
                    ((9 + dir_line * 4) as usize, (9 + dir_col * 4) as usize)
                }
                game::TypeOfParty::Standard => ia(game, (ztable, hash), depth_max, start_time),
            }
        }
        _ => {
            let ret = ia(game, (ztable, hash), depth_max, start_time);
            ret
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_ia(
        white_pos: Vec<(usize, usize)>,
        black_pos: Vec<(usize, usize)>,
        actual: Option<bool>,
        actual_catch: &mut isize,
        opp_catch: &mut isize,
        depth_max: &i8,
        expected_result: (usize, usize),
    ) -> bool {
        let mut board = [[None; SIZE_BOARD]; SIZE_BOARD];
        white_pos
            .iter()
            .for_each(|&(x, y)| board[x][y] = Some(true));
        black_pos
            .iter()
            .for_each(|&(x, y)| board[x][y] = Some(false));
        let mut score_board = heuristic::evaluate_board(&mut board);
        let mut tt = zobrist::initialize_transposition_table();
        let ztable = zobrist::init_zboard();
        let mut hash = zobrist::board_to_zhash(&mut board, &ztable);
        println!("// Initial configuration:");
        for i in 0..19 {
            print!("// ");
            for j in 0..19 {
                match board[j][i] {
                    Some(true) => print!("⊖"),
                    Some(false) => print!("⊕"),
                    None => print!("_"),
                }
            }
            println!();
        }
        let (_, (x, y)) = mtdf(
            &mut board,
            &ztable,
            &mut hash,
            &mut tt,
            actual,
            actual_catch,
            opp_catch,
            &mut MAX_INFINITY,
            depth_max,
            0,
            &mut score_board,
        );
        println!("// Result IA ({},{}) :", x, y);
        for i in 0..19 {
            print!("// ");
            for j in 0..19 {
                if i == y && j == x && actual == Some(false) {
                    print!("⊛");
                } else if i == y && j == x && actual == Some(true) {
                    print!("⊙");
                } else {
                    match board[j][i] {
                        Some(true) => print!("⊖"),
                        Some(false) => print!("⊕"),
                        None => print!("_"),
                    }
                }
            }
            println!();
        }
        expected_result == (x, y)
    }

    //    #[test]
    //    fn test_ia_board_00() {
    //        let black_pos = vec![
    //            (6, 8),
    //            (10, 8),
    //            (7, 9),
    //            (9, 9),
    //            (6, 10),
    //            (8, 10),
    //            (10, 10),
    //            (5, 11),
    //            (7, 11),
    //            (10, 11),
    //            (7, 12),
    //            (10, 12),
    //            (10, 13),
    //        ];
    //        let white_pos = vec![
    //            (5, 7),
    //            (7, 7),
    //            (9, 7),
    //            (11, 7),
    //            (8, 6),
    //            (9, 8),
    //            (8, 9),
    //            (7, 10),
    //            (10, 9),
    //            (9, 11),
    //            (11, 11),
    //            (4, 12),
    //            (10, 14),
    //        ];
    //        let actual = Some(true);
    //        let mut actual_catch = 1isize;
    //        let mut opp_catch = 1isize;
    //        let depth_max = 5i8;
    //        let expected_result = (0, 0);
    //        assert!(test_ia(
    //            black_pos,
    //            white_pos,
    //            actual,
    //            &mut actual_catch,
    //            &mut opp_catch,
    //            &depth_max,
    //            expected_result,
    //        ));
    //    }
}
