// use super::super::model::point;
// use rand::distributions::{Distribution, Uniform};
// use rand::thread_rng;
use super::super::checks::after_turn_check::DIRECTIONS;
use super::super::checks::capture;
use super::super::model::board::Board;
use super::super::model::bool_option::get_opp;
use super::super::model::game;
use super::super::model::history;
use super::super::model::params;
use super::super::model::params::{ParamsIA, ThreadPool};
use super::super::model::score_board::ScoreBoard;
use super::super::render::board::SIZE_BOARD;
use super::handle_board::{
    board_state_win, change_board, change_board_hint, find_continuous_threats, get_space,
    remove_last_pawn, remove_last_pawn_hint,
};
// use super::super::model::player;
use super::heuristic;
use super::zobrist;
use rand::seq::SliceRandom;
// use std::sync::mpsc::Sender;
use std::time;
// use super::super::player;

const MIN_INFINITY: i64 = i64::min_value() + 1;
const MAX_INFINITY: i64 = i64::max_value();
// const DEPTH_THREATS: i8 = 10;
const LIMIT_DURATION: time::Duration = time::Duration::from_millis(495);
const SILENT_MOVE_SCORE: i64 = 1000000;

macro_rules! get_usize {
    ($e:expr) => {
        match $e {
            Some(true) => 0,
            Some(false) => 1,
            _ => unreachable!(),
        }
    };
}
const PRINT_LETTER: [&str; 19] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
];

fn find_winning_align(
    board: &mut Board,
    score_board: &mut ScoreBoard,
    actual: Option<bool>,
    check_capture: bool,
) -> bool {
    for line in 0..SIZE_BOARD {
        for col in 0..SIZE_BOARD {
            if board.get_pawn(line, col) == actual {
                for dir in 0..4 {
                    match score_board.get(line, col, dir) {
                        (a, _, _) if a >= 5 => {
                            let mut align = Vec::with_capacity(10);
                            let (dx, dy) = DIRECTIONS[dir];
                            if !check_capture {
                                return true;
                            }
                            align.push((line as isize, col as isize));
                            for way in [-1, 1].iter() {
                                let mut new_x = line as isize + dx * way;
                                let mut new_y = col as isize + dy * way;
                                loop {
                                    new_x += way * dx;
                                    new_y += way * dy;
                                    match board.get(new_x as usize, new_y as usize) {
                                        Some(a) if a == actual => align.push((new_x, new_y)),
                                        _ => break,
                                    }
                                }
                            }
                            return !capture::can_capture_vec_hint(board, score_board, align);
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
    // board: &mut Board,
    // score_board: &mut ScoreBoard,
    // zhash: &mut u64,
    // counter_tree: &mut u64,
    // start_time: &time::Instant,
    params: &mut ParamsIA,
    // table: &[[[u64; 2]; SIZE_BOARD]; SIZE_BOARD],
    current_depth: &mut i8,
    actual: Option<bool>,
    actual_catch: &mut isize,
    opp_catch: &mut isize,
    alpha: &mut i64,
    beta: &mut i64,
    color: &mut i8,
    depth_max: &i8,
    htable: &mut [[[i32; SIZE_BOARD]; SIZE_BOARD]; 2],
) -> Option<(i64, Option<(usize, usize)>)> {
    // println!("entered: {}", counter_tree);
    if params.check_timeout() {
        return None;
    }
    let mut tte = zobrist::retrieve_tt_from_hash(&params.zhash);
    let alpha_orig = *alpha;
    params.counter_tree += 1;
    if tte.is_valid && tte.depth >= *depth_max - *current_depth {
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
        return Some((
            -heuristic::INSTANT_WIN * ((*depth_max - *current_depth) as i64 + 1),
            None,
        ));
    } else if find_winning_align(
        &mut params.board,
        &mut params.score_board,
        get_opp(actual),
        true,
    ) {
        if *current_depth == 0 {
            println!("WIIIINNNNNNING ALIGN 11111 - {}", params.counter_tree);
        }
        return Some((
            -heuristic::INSTANT_WIN * ((*depth_max - *current_depth) as i64 + 1),
            None,
        ));
    } else if find_winning_align(&mut params.board, &mut params.score_board, actual, false) {
        if *current_depth == 0 {
            println!("WIIIINNNNNNING ALIGN 22222222 - {}", params.counter_tree);
        }
        return Some((
            heuristic::INSTANT_WIN * ((*depth_max - *current_depth) as i64 + 1),
            None,
        ));
    }
    if *current_depth == *depth_max {
        //        if let Some((_, _)) = find_continuous_threats(
        //            &mut params.board,
        //            &mut params.score_board,
        //            actual,
        //            actual_catch,
        //            opp_catch,
        //            &mut 4,
        //            &mut 0,
        //            true,
        //        ) {
        //            println!("yo");
        //            return Some((
        //                -heuristic::INSTANT_WIN * ((*depth_max - *current_depth) as i64 + 1),
        //                None,
        //            ));
        //        }
        let weight = heuristic::first_heuristic_hint(
            &mut params.board,
            &mut params.score_board,
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
            if params.board.get_pawn(line, col) == None {
                let removed = change_board(
                    &mut params.board,
                    &mut params.score_board,
                    line,
                    col,
                    actual,
                    &mut params.zhash,
                );
                *actual_catch += removed.len() as isize;

                // Recurse
                let value = ab_negamax(
                    params,
                    &mut (*current_depth + 1),
                    get_opp(actual),
                    opp_catch,
                    actual_catch,
                    &mut (-*beta),
                    &mut (-*alpha),
                    &mut (-*color),
                    depth_max,
                    htable,
                );
                //                println!("here5");

                *actual_catch -= removed.len() as isize;
                remove_last_pawn(
                    &mut params.board,
                    &mut params.score_board,
                    line,
                    col,
                    actual,
                    removed,
                    &mut params.zhash,
                );
                match value {
                    // None => return None,
                    None => {
                        return None;
                    }
                    Some((recursed_score, _)) => {
                        let x = -recursed_score;

                        if x >= *beta {
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
        let mut available_positions = get_space(
            &mut params.board,
            &mut params.score_board,
            actual,
            *actual_catch,
        );
        let mut silent_moves = available_positions.split_off(
            available_positions
                .iter()
                .position(|&x| x.2 < SILENT_MOVE_SCORE)
                .unwrap() as usize,
        );
        history::sort_silent_moves(&htable, get_usize!(actual), &mut silent_moves);
        let len_available_positions = available_positions.len();
        available_positions.append(&mut silent_moves);
        // println!("\nCounter-tree: {}|d:{}", counter_tree,current_depth);
        // available_positions.iter().for_each(|&(x,y,z)|{
        //     print!("[({}:{})|{}]", x,  y, z);
        // });
        // println!("");
        //        println!("here8");
        let mut tmp_curr_depth = *current_depth + 1;
        // let calc_depth = cmp::min(((*depth_max - *current_depth) / 2) + *current_depth, *depth_max);
        for (index, &(line, col, _)) in available_positions.iter().enumerate() {
            if *depth_max >= 6
                && *current_depth > 2
                && depth_max - *current_depth < 4
                && (depth_max - *current_depth) * 8 < index as i8
                && best_score > -heuristic::INSTANT_WIN
            {
                break;
            }
            // if params.check_timeout() {
            //     return None
            // }
            let removed = change_board(
                &mut params.board,
                &mut params.score_board,
                line,
                col,
                actual,
                &mut params.zhash,
            );
            *actual_catch += removed.len() as isize;

            // if index == 5 {
            //     tmp_curr_depth = cmp::min(*current_depth + 3, *depth_max);
            // }

            // Recurse
            //            println!("here9");
            let value = ab_negamax(
                params,
                &mut tmp_curr_depth,
                get_opp(actual),
                opp_catch,
                actual_catch,
                &mut (-*beta),
                &mut (-*alpha),
                &mut (-*color),
                depth_max,
                htable,
            );

            *actual_catch -= removed.len() as isize;
            remove_last_pawn(
                &mut params.board,
                &mut params.score_board,
                line,
                col,
                actual,
                removed,
                &mut params.zhash,
            );
            //            println!("here10");
            match value {
                None => {
                    // if *current_depth == 1 || *current_depth == 0  || *current_depth == 2 {
                    // continue ;
                    // } else {
                    return None;
                    // }
                }
                Some((recursed_score, _)) => {
                    //println!("{}/{}|score:{}", PRINT_LETTER[line], col, -recursed_score);
                    let x = -recursed_score;
                    if x > best_score {
                        best_score = x;
                        best_move = Some((line, col));
                    }
                    if x > *alpha {
                        *alpha = x;
                        best_move = Some((line, col));
                    }

                    if *alpha >= *beta {
                        // History table cutoff
                        if index >= len_available_positions {
                            history::update_htable(
                                htable,
                                &available_positions[len_available_positions..index],
                                get_usize!(actual),
                                &available_positions[index],
                                current_depth,
                            );
                        }
                        best_score = *alpha;
                        best_move = Some((line, col));
                        break;
                    }
                }
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
    tte.key = params.zhash;
    tte.value = best_score;
    tte.r#move = best_move;
    tte.depth = *depth_max - *current_depth;
    zobrist::store_tt_entry(&mut params.zhash, tte);

    Some((best_score, best_move))
}

fn mtdf(
    params: &mut ParamsIA,
    htable: &mut [[[i32; SIZE_BOARD]; SIZE_BOARD]; 2],
) -> Option<(i64, (usize, usize))> {
    let mut g = params.f;
    let mut ret = (0, (0, 0));
    let mut upperbnd = MAX_INFINITY;
    let mut lowerbnd = MIN_INFINITY;
    //    let board_save = params.board;
    //    let score_board_save = params.score_board;

    while lowerbnd < upperbnd {
        let mut depth_max = params.depth_max;
        let mut actual_catch2 = params.actual_catch;
        let mut opp_catch2 = params.opp_catch;

        let mut beta = if g == lowerbnd { g + 1 } else { g };

        let values: Option<(i64, Option<(usize, usize)>)> = ab_negamax(
            params,
            &mut 0,
            params.actual,
            &mut actual_catch2,
            &mut opp_catch2,
            &mut (beta - 1),
            &mut beta,
            &mut 1,
            &mut depth_max,
            htable,
        );
        //        if actual_catch2 != params.actual_catch || opp_catch2 != params.opp_catch {
        //            println!("aye");
        //        }
        //        if board_save != params.board {
        //            println!("ouille");
        //        }
        //        if score_board_save != params.score_board {
        //            println!("aille");
        //        }
        match values {
            None => return None,
            Some((score, r#move)) => {
                match r#move {
                    None => return None,
                    Some(r#move) => {
                        ret = (score, r#move);
                        g = score;
                        if g < beta {
                            upperbnd = g;
                        } else {
                            lowerbnd = g;
                        }
                    }
                }
            }
        }
    }
    Some(ret)
}

pub fn iterative_deepening_mtdf(params: &mut ParamsIA, mainloop: bool) -> (i64, (usize, usize)) {
    let mut ret = (MIN_INFINITY, (0, 0));
    let beta = params.beta;
    let actual_catch = params.actual_catch;
    let opp_catch = params.opp_catch;
    let mut htable = history::initialize_htable();
    let actual = params.actual;

    for d in (2..(params.depth_max + 0)).step_by(2) {
        // Below, their existence is justified (checks still needed for beta)
        params.counter_tree = 0;
        params.depth_max = d;
        params.beta = beta;
        params.actual_catch = actual_catch;
        params.opp_catch = opp_catch;

        let stime_mtdf = time::Instant::now();
        if unsafe { params::STOP_THREADS }
            || stime_mtdf.duration_since(params.start_time) >= LIMIT_DURATION
        {
            break;
        }

        let tmp_ret = mtdf(params, &mut htable);
        match tmp_ret {
            None => {
                let available_positions = get_space(
                    &mut params.board,
                    &mut params.score_board,
                    actual,
                    actual_catch,
                )
                .pop()
                .unwrap();
                if ret == (MIN_INFINITY, (0, 0)) {
                    ret = (
                        available_positions.2,
                        (available_positions.0, available_positions.1),
                    );
                }
                break;
            }
            Some((score, r#move)) => {
                let ndtime_mtdf = time::Instant::now();
                if mainloop {
                    println!(
                        "Depth: [{}] | Nb. moves: [{}] | Nb. moves/s: [{}]",
                        d,
                        params.counter_tree,
                        (params.counter_tree as f64
                            / ndtime_mtdf.duration_since(stime_mtdf).as_secs_f64())
                        .floor()
                    );
                }
                params.f = score;
                ret = (score, r#move);
            } // params.f = score;
        }
    }
    ret
}

fn ia(
    game: &mut game::Game,
    hash: u64,
    depth_max: &i8,
    start_time: &time::Instant,
    threadpool: &ThreadPool,
) -> (usize, usize) {
    let mut board: Board = game.board.into();
    let mut params = ParamsIA {
        score_board: heuristic::evaluate_board(&mut board),
        board: board,
        zhash: hash,
        current_depth: 0,
        actual: game.player_to_pawn(),
        actual_catch: game.get_actual_player().nb_of_catch,
        opp_catch: game.get_opponent().nb_of_catch,
        alpha: MIN_INFINITY,
        beta: MAX_INFINITY,
        color: 0,
        depth_max: *depth_max,
        counter_tree: 0,
        start_time: *start_time,
        f: 0,
        counter: 0,
    };
    //println!("---------------------------------------------------------------------------");

    // Spawn 3 threads for parallel execution
    for _ in 0..3 {
        // let tx_tmp = threadpool.tx.clone();
        let mut params_tmp = params.clone();
        let _ = threadpool.pool.spawn(move || {
            iterative_deepening_mtdf(&mut params_tmp, false);
        });
    }
    // Main thread execution
    iterative_deepening_mtdf(&mut params, true).1
}

pub fn get_ia(
    game: &mut game::Game,
    depth_max: &i8,
    start_time: &time::Instant,
    threadpool: &ThreadPool,
) -> (usize, usize) {
    let hash: u64 = zobrist::board_to_zhash(&game.board);
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
                game::TypeOfParty::Standard => ia(game, hash, depth_max, start_time, threadpool),
            }
        }
        _ => {
            let ret = ia(game, hash, depth_max, start_time, threadpool);
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
        zobrist::init_zboard();
        let mut hash = zobrist::board_to_zhash(&mut bboard);
        println!("// Initial configuration:");
        board.print();
        // let stime_mtdf = time::Instant::now();
        let mut params = ParamsIA {
            score_board: score_board,
            board: board,
            zhash: hash,
            current_depth: 0,
            actual: actual,
            actual_catch: *actual_catch,
            opp_catch: *opp_catch,
            alpha: MIN_INFINITY,
            beta: MAX_INFINITY,
            color: 0,
            depth_max: *depth_max,
            counter_tree: 0,
            start_time: time::Instant::now(),
            f: 0,
            counter: 0,
        };
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
        // let mut counter_tree:u64 = 0;
        let mut htable = history::initialize_htable();
        let result = mtdf(&mut params, &mut htable);
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
                            match board.get_pawn(j, i) {
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
