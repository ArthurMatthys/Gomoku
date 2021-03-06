use super::super::checks::after_turn_check::DIRECTIONS;
use super::super::checks::capture;
use super::super::model::board::Board;
use super::super::model::bool_option::get_opp;
use super::super::model::game;
use super::super::model::history;
use super::super::model::params;
use super::super::model::params::ParamsIA;
use super::super::render::board::SIZE_BOARD;
use super::handle_board::{
    change_board, find_continuous_threats, get_space, null_move_heuristic, remove_last_pawn,
};
use super::heuristic;
use super::zobrist;
use rand::seq::SliceRandom;
use rayon::prelude::*;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::time;

const MIN_INFINITY: i64 = i64::min_value() + 1;
const MAX_INFINITY: i64 = i64::max_value();
const LIMIT_DURATION: time::Duration = time::Duration::from_millis(480);
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

pub fn find_winning_align(
    params: &mut ParamsIA,
    actual: Option<bool>,
    check_capture: bool,
) -> bool {
    for line in 0..SIZE_BOARD {
        for col in 0..SIZE_BOARD {
            if params.board.get_pawn(line, col) == actual {
                for dir in 0..4 {
                    match params.score_board.get(line, col, dir) {
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
                                    match params.board.get(new_x as usize, new_y as usize) {
                                        Some(a) if a == actual => align.push((new_x, new_y)),
                                        _ => break,
                                    }
                                }
                            }
                            return !capture::can_capture_vec_hint(
                                &mut params.board,
                                &mut params.score_board,
                                align,
                            );
                        }
                        _ => (),
                    }
                }
            }
        }
    }
    false
}

fn ab_negamax(
    params: &mut ParamsIA,
    current_depth: &mut i8,
    actual: Option<bool>,
    actual_catch: &mut isize,
    opp_catch: &mut isize,
    alpha: &mut i64,
    beta: &mut i64,
    color: &mut i8,
    depth_max: &i8,
    htable: &mut [[[i32; SIZE_BOARD]; SIZE_BOARD]; 2],
    mainloop: bool,
) -> Option<(i64, Option<(usize, usize)>)> {
    if params.check_timeout(mainloop) {
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

    if *opp_catch >= 5 {
        return Some((
            -heuristic::INSTANT_WIN * ((*depth_max - *current_depth) as i64 + 1),
            None,
        ));
    } else if find_winning_align(params, get_opp(actual), true) {
        return Some((
            -heuristic::INSTANT_WIN * ((*depth_max - *current_depth) as i64 + 1),
            None,
        ));
    } else if find_winning_align(params, actual, false) {
        return Some((
            heuristic::INSTANT_WIN * ((*depth_max - *current_depth) as i64 + 1),
            None,
        ));
    }
    if *current_depth == *depth_max {
        // if mainloop {
        //     if let Some((_, _)) = find_continuous_threats(
        //         params,
        //         actual,
        //         actual_catch,
        //         opp_catch,
        //         &mut 4,
        //         &mut 0,
        //         true,
        //         mainloop
        //     ) {
        //         return Some((
        //             heuristic::INSTANT_WIN * ((*depth_max - *current_depth) as i64 + 1),
        //             None,
        //         ));
        //     } else if let Some((_, _)) = find_continuous_threats(
        //         params,
        //         get_opp(actual),
        //         opp_catch,
        //         actual_catch,
        //         &mut 4,
        //         &mut 0,
        //         true,
        //         mainloop
        //     ) {
        //         return Some((
        //             -heuristic::INSTANT_WIN * ((*depth_max - *current_depth) as i64 + 1),
        //             None,
        //         ));
        //     }
        // }

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

    // Otherwise bubble up values from below
    let mut best_move: Option<(usize, usize)> = None;
    let mut best_score = MIN_INFINITY;
    let mut trig = false;

    if tte.is_valid && tte.depth >= *depth_max - *current_depth {
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
                    mainloop,
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
                match value {
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
            }
        }
    }

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
        // Late move reduction
        if *current_depth >= 6 {
            let silent_cutoff: usize;
            if silent_moves.len() > ((depth_max - *current_depth) * 8) as usize {
                silent_cutoff = ((depth_max - *current_depth) * 8) as usize;
            } else {
                silent_cutoff = silent_moves.len() as usize;
            }
            silent_moves = silent_moves[0..silent_cutoff].to_vec()
        }
        let len_available_positions = available_positions.len();
        available_positions.append(&mut silent_moves);
        let mut tmp_curr_depth: i8 = *current_depth + 1_i8;

        for (index, &(line, col, _)) in available_positions.iter().enumerate() {
            if params.check_timeout(mainloop) {
                return None;
            }
            let removed = change_board(
                &mut params.board,
                &mut params.score_board,
                line,
                col,
                actual,
                &mut params.zhash,
            );
            *actual_catch += removed.len() as isize;

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
                mainloop,
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

            match value {
                None => {
                    return None;
                }
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

                    if *alpha >= *beta {
                        // History table cutoff
                        if index >= len_available_positions {
                            history::update_htable(
                                htable,
                                &available_positions[len_available_positions..index],
                                get_usize!(actual),
                                &available_positions[index],
                                &current_depth,
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
    mainloop: bool,
) -> Option<(i64, (usize, usize))> {
    let mut g = params.f;
    let mut ret = (0, (0, 0));
    let mut upperbnd = MAX_INFINITY;
    let mut lowerbnd = MIN_INFINITY;

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
            mainloop,
        );

        match values {
            None => return None,
            Some((score, r#move)) => match r#move {
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
            },
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

        let tmp_ret = mtdf(params, &mut htable, mainloop);
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
            }
        }
    }
    ret
}

fn ia(
    game: &mut game::Game,
    hash: u64,
    depth_max: &i8,
    start_time: &time::Instant,
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
    let mut depth_max_threat = 10;

    //Spawn 4 threads for parallel execution
    let (sender, receiver): (
        Sender<(i64, (usize, usize))>,
        Receiver<(i64, (usize, usize))>,
    ) = channel();
    (0..4).into_par_iter().for_each_with(sender, |s, i| {
        let mut params_tmp = params.clone();
        if i == 3 {
            s.send(iterative_deepening_mtdf(&mut params_tmp, true))
                .unwrap();
        } else {
            iterative_deepening_mtdf(&mut params_tmp, false);
        }
    });
    let ret: Vec<(i64, (usize, usize))> = receiver.into_iter().collect();
    // let ret = vec![iterative_deepening_mtdf(&mut params, true)];
    if let Some(_) = null_move_heuristic(&mut params) {
        ret[0].1
    } else {
        if let Some((x, y)) =
            find_continuous_threats(&mut params, &mut depth_max_threat, &mut 0, true, true)
        {
            return (x, y);
        }
        ret[0].1
    }
}

pub fn get_ia(game: &mut game::Game, depth_max: &i8, start_time: &time::Instant) -> (usize, usize) {
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
                game::TypeOfParty::Standard => ia(game, hash, depth_max, start_time),
            }
        }
        _ => {
            let ret = ia(game, hash, depth_max, start_time);
            ret
        }
    }
}
