// use super::super::model::point;
// use rand::distributions::{Distribution, Uniform};
// use rand::thread_rng;
use super::super::checks::capture;
use super::super::model::game;
use super::super::model::board::Board;
use super::super::model::score_board::ScoreBoard;
use super::super::render::board::SIZE_BOARD;
use super::handle_board::{
    board_state_win, change_board, change_board_hint, find_continuous_threats, get_space,
    null_move_heuristic, remove_last_pawn, remove_last_pawn_hint, print_board
};
// use super::super::model::player;
use super::heuristic;
use super::zobrist;
use rand::seq::SliceRandom;
use std::time;
use std::cmp;
// use super::super::player;

const MIN_INFINITY: i64 = i64::min_value() + 1;
const MAX_INFINITY: i64 = i64::max_value();
const DEPTH_THREATS: i8 = 10;
const LIMIT_DURATION: time::Duration = time::Duration::from_millis(495);

macro_rules! get_opp {
    ($e:expr) => {
        match $e {
            Some(a) => Some(!a),
            _ => unreachable!(),
        }
    };
}

fn find_winning_align(
    board: &mut Board,
    score_board: &mut ScoreBoard,
    actual: Option<bool>,
) -> bool {
    for line in 0..SIZE_BOARD {
        for col in 0..SIZE_BOARD {
            if board.get_pawn(line, col) == actual {
                for dir in 0..4 {
                    match score_board.get(line, col, dir) {
                        (a, _, _) if a >= 5 => {
                            return true;
                        }
                        _ => (),
                    }
                }
            }
        }
    }
    false
}

// negamax_try
fn ab_negamax(
    board: &mut Board,
    table: &[[[u64; 2]; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut ScoreBoard,
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
    counter_tree: &mut u64,
    start_time: &time::Instant
) -> Option<(i64, Option<(usize, usize)>)> {
    // println!("entered: {}", counter_tree);
    if time::Instant::now().duration_since(*start_time) >= LIMIT_DURATION {
        // println!("cutoff: {}", counter_tree);
        // println!("DAAAAAMMMNN: {}", current_depth);
        //
        return None;
    }
//    println!("here1");
    let mut tte = zobrist::retrieve_tt_from_hash(tt, zhash);
    let alpha_orig = *alpha;
    *counter_tree += 1;
    if tte.is_valid && tte.depth == *depth_max - *current_depth {
        if tte.r#type == zobrist::TypeOfEl::Exact {
            return Some((tte.value, tte.r#move));
        } else if tte.r#type == zobrist::TypeOfEl::Lowerbound {
            *alpha = i64::max(*alpha, tte.value);
        } else if tte.r#type == zobrist::TypeOfEl::Upperbound {
            *beta = i64::min(*beta, tte.value);
        }

        if *alpha >= *beta {
            return Some((tte.value, tte.r#move));
        }
    }

//    println!("here2");
    if *opp_catch >= 5 {
        return Some((-heuristic::INSTANT_WIN * ((*depth_max - *current_depth) as i64 + 1), None));
    } else if find_winning_align(board, score_board, actual) {
        return Some((heuristic::INSTANT_WIN * ((*depth_max - *current_depth) as i64 + 1), None));
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
        return Some((weight, None));
    }

//    println!("here3");
    // Otherwise bubble up values from below
    let mut best_move: Option<(usize, usize)> = None;
    let mut best_score = MIN_INFINITY;
    let mut trig = false;

    if tte.is_valid && tte.depth >= *depth_max - *current_depth {
        // println!("rentré");
//        println!("here4");
        if let Some((line, col)) = tte.r#move {
            if board.get_pawn(line, col) == None {
                let removed = change_board(board, score_board, line, col, actual, table, zhash);
                *actual_catch += removed.len() as isize;

                // Recurse
                let value = ab_negamax(
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
                    counter_tree,
                    start_time
                );
//                println!("here5");

                match value {
                    // None => return None,
                    None => { 
                        *actual_catch -= removed.len() as isize;
                        remove_last_pawn(board, score_board, line, col, actual, removed, table, zhash);
                        return None
                    },
                    Some((recursed_score, _)) => {
                        let x = -recursed_score;
        
                        *actual_catch -= removed.len() as isize;
                        remove_last_pawn(board, score_board, line, col, actual, removed, table, zhash);
        
                        if x >= *beta {
                            // println!("rentré_cutoff");
                            best_score = x;
                            best_move = tte.r#move;
                            trig = true;
                        }
                    }
                }
//                println!("here6");
                // else {
                //     best_score = MIN_INFINITY;
                //     best_move = None;
                // }
            }
        }
    }

//    println!("here7");
    if !trig {
        // Collect moves
        let available_positions = get_space(board, score_board, actual, *actual_catch);
        // println!("\nCounter-tree: {}|d:{}", counter_tree,current_depth);
        // available_positions.iter().for_each(|&(x,y,z)|{
        //     print!("[({}:{})|{}]", x,  y, z);
        // });
        // println!("");
//        println!("here8");
        let mut tmp_curr_depth = *current_depth + 1;
        // let calc_depth = cmp::min(((*depth_max - *current_depth) / 2) + *current_depth, *depth_max);
        for (index, &(line, col, _)) in available_positions.iter().enumerate() {
            let removed = change_board(board, score_board, line, col, actual, table, zhash);
            *actual_catch += removed.len() as isize;

            // if index == 5 {
            //     tmp_curr_depth = cmp::min(*current_depth + 3, *depth_max);
            // }

            // Recurse
//            println!("here9");
            let value = ab_negamax(
                board,
                table,
                score_board,
                zhash,
                tt,
                // &mut (*current_depth + 1),
                &mut tmp_curr_depth,
                get_opp!(actual),
                opp_catch,
                actual_catch,
                &mut (-*beta),
                &mut (-*alpha),
                &mut (-*color),
                depth_max,
                counter_tree,
                start_time
            );

//            println!("here10");
            match value {
                None => { 
                    *actual_catch -= removed.len() as isize;
                    remove_last_pawn(board, score_board, line, col, actual, removed, table, zhash);
                    // if *current_depth == 1 || *current_depth == 0  || *current_depth == 2 {
                        // continue ;
                    // } else {
                        return None
                    // }
                },
                Some((recursed_score, _)) => {
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
//            println!("here11");
        }
    }

    if best_score <= alpha_orig {
        tte.r#type = zobrist::TypeOfEl::Upperbound;
    } else if best_score >= *beta {
        tte.r#type = zobrist::TypeOfEl::Lowerbound;
    } else {
        tte.r#type = zobrist::TypeOfEl::Exact;
    }
//    println!("here12");
    tte.is_valid = true;
    tte.key = *zhash;
    tte.value = best_score;
    tte.r#move = best_move;
    tte.depth = *depth_max - *current_depth;
    zobrist::store_tt_entry(tt, zhash, tte);

    Some((best_score, best_move))
}

fn mtdf(
    board: &mut Board,
    table: &[[[u64; 2]; SIZE_BOARD]; SIZE_BOARD],
    zhash: &mut u64,
    tt: &mut Vec<zobrist::TT>,
    actual: Option<bool>,
    actual_catch: &mut isize,
    opp_catch: &mut isize,
    beta: &mut i64,
    depth_max: &i8,
    firstguess: i64,
    score_board: &mut ScoreBoard,
    counter_tree: &mut u64,
    start_time: &time::Instant
) -> Option<(i64, (usize, usize))> {
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
        // if time::Instant::now().duration_since(*start_time) >= LIMIT_DURATION {
        //     println!("Mtd-f: break before");
        //     return None;
        // }
        // *beta = i64::max(g, lowerbnd + 1);
        let values: Option<(i64, Option<(usize, usize)>)> = ab_negamax(
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
            counter_tree,
            start_time
        );
        match values {
            // None => { println!("DEBUUGG: d:{}", depth_max); return None },
            None => { return None },
            Some((score, r#move)) => {
                ret = (score, r#move.unwrap());
                g = score;
                if g < *beta {
                    upperbnd = g;
                } else {
                    lowerbnd = g;
                }
            }
        }
        // if time::Instant::now().duration_since(*start_time) >= LIMIT_DURATION {
        //     println!("Mtd-f: break after");
        //     return None;
        // }
    }
    Some(ret)
}

pub fn iterative_deepening_mtdf(
    board: &mut Board,
    score_board: &mut ScoreBoard,
    table: &[[[u64; 2]; SIZE_BOARD]; SIZE_BOARD],
    zhash: &mut u64,
    tt: &mut Vec<zobrist::TT>,
    actual: Option<bool>,
    actual_catch: &mut isize,
    opp_catch: &mut isize,
    beta: &mut i64,
    depth_max: &i8,
    game: &mut game::Game,
    start_time: &time::Instant,
    null_move: bool
) -> (i64,(usize, usize)) {
//    println!("========================");
//    println!("Entering Iterative MTDF");
    // print_board(board);
    // println!();
    let mut ret = (MIN_INFINITY,(0, 0));
    let mut f = 0;
    // let mut f = match actual {
    //     Some(true) => game.firstguess.0,
    //     Some(false) => game.firstguess.1,
    //     None => unreachable!(),
    // };
    // let limit_duration = time::Duration::from_millis(480);

    // println!("before- f: {}", f);
    // for d in [2, 3, 5].iter() {
    for d in (2..(depth_max + 1)).step_by(2) {
        let mut beta2 = *beta;
        let mut actual_catch2 = *actual_catch;
        let mut opp_catch2 = *opp_catch;

        // for d in (1..DEPTH_MAX).step_by(2) {
        //    println!("debug: {}|{}|{}|{}|{}|{}|", *alpha, *beta, actual_catch2, opp_catch2, d, *zhash);
        let mut counter_tree:u64 = 0;
        let stime_mtdf = time::Instant::now();
        if stime_mtdf.duration_since(*start_time) >= LIMIT_DURATION {
            break;
        }
        let tmp_ret = mtdf(
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
            &mut counter_tree,
            start_time
        );
        // println!("middle_mtdf");
        // print_board(board);
        // println!();
        match tmp_ret {
            None => break,
            Some((score, r#move)) => {   
                // println!("move_iterative_deepening_mtdf: [score:{}|{}/{}]",score, r#move.0,r#move.1);
                let ndtime_mtdf = time::Instant::now();
                if !null_move {
                    println!("Depth: [{}] | Nb. moves: [{}] | Nb. moves/s: [{}]", d, counter_tree, (counter_tree as f64 / ndtime_mtdf.duration_since(stime_mtdf).as_secs_f64()).floor());
                }
                ret = (score,r#move);
                f = score;
            }
        }
        // println!("debug2: {}|{}|{}|{}|{}|{}|", *alpha, *beta, actual_catch2, opp_catch2, d, *zhash);
    }
    // match actual {
    //     Some(true) => game.firstguess.0 = f,
    //     Some(false) => game.firstguess.1 = f,
    //     None => unreachable!(),
    // };
    // println!("after- f: {}", f);
    ret
}

fn ia(
    game: &mut game::Game,
    (table, mut hash): (&[[[u64; 2]; SIZE_BOARD]; SIZE_BOARD], u64),
    depth_max: &i8,
    start_time: &time::Instant
) -> (usize, usize) {
    // let end = time::Instant::now();
    // println!("enter_ia: {}",end.duration_since(*start_time).as_secs_f64());
    let mut player_catch = game.get_actual_player().nb_of_catch;
    let mut opponent_catch = game.get_opponent().nb_of_catch;
    let mut board: Board = game.board.into();
    let mut score_board = heuristic::evaluate_board(&mut board);
    let pawn = game.player_to_pawn();
    let mut tt = zobrist::initialize_transposition_table();
    // let end = time::Instant::now();
    // println!("after_initialization: {}",end.duration_since(*start_time).as_secs_f64());
    
    // let end = time::Instant::now();
    // println!("before_null: {}",end.duration_since(*start_time).as_secs_f64());
    if let Some((x, y)) = null_move_heuristic(
        &mut board,
        &mut score_board,
        pawn,
        &mut opponent_catch,
        &mut player_catch,
        &(table, hash),
        start_time,
        game
    ) {
        println!("Answer null move ({},{})", x, y);
        return (x, y);
    }
    // let end = time::Instant::now();
    // println!("after_null: {}",end.duration_since(*start_time).as_secs_f64());
    // let end = time::Instant::now();
    // println!("before continuous: {}",end.duration_since(*start_time).as_secs_f64());
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
    // let end = time::Instant::now();
    // println!("after_continuous: {}",end.duration_since(*start_time).as_secs_f64());
    // let end = time::Instant::now();
    // println!("before mtdf: {}",end.duration_since(*start_time).as_secs_f64());
    // println!("NEW COUP");
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
        start_time,
        false
    ).1
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
        let mut bboard = [[None; SIZE_BOARD]; SIZE_BOARD];
        white_pos
            .iter()
            .for_each(|&(x, y)| bboard[x][y] = Some(true));
        black_pos
            .iter()
            .for_each(|&(x, y)| bboard[x][y] = Some(false));
        let mut board: Board = bboard.into();
        let mut score_board = heuristic::evaluate_board(&mut board);
        let mut tt = zobrist::initialize_transposition_table();
        let ztable = zobrist::init_zboard();
        let mut hash = zobrist::board_to_zhash(&mut bboard, &ztable);
        println!("// Initial configuration:");
        board.print();
//        for i in 0..19 {
//            print!("// ");
//            for j in 0..19 {
//                match board[j][i] {
//                    Some(true) => print!("⊖"),
//                    Some(false) => print!("⊕"),
//                    None => print!("_"),
//                }
//            }
//            println!();
//        }
        let mut counter_tree:u64 = 0;
        let stime_mtdf = time::Instant::now();
        let result = mtdf(
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
            &mut counter_tree,
            &stime_mtdf
        );
        match result {
            None => false,
            Some((_, (x, y))) => {
                println!("// Result IA ({},{}) :", x, y);
                for i in 0..19 {
                    print!("// ");
                    for j in 0..19 {
                        if i == y && j == x && actual == Some(false) {
                            print!("⊛");
                        } else if i == y && j == x && actual == Some(true) {
                            print!("⊙");
                        } else {
                            match board.get_pawn(j, i){
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
        }
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
