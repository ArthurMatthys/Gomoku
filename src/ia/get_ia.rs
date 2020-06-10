// use super::super::model::point;
// use rand::distributions::{Distribution, Uniform};
// use rand::thread_rng;
use super::super::checks::capture;
use super::super::model::game;
use super::super::render::board::SIZE_BOARD;
use super::handle_board::{board_state_win, change_board, get_space, remove_last_pawn};
// use super::super::model::player;
use super::heuristic;
use super::zobrist;
use rand::seq::SliceRandom;
// use super::super::player;
use std::thread::sleep;
use std::time::Duration;

const DEPTH_MAX: i8 = 4;
const MIN_INFINITY: i64 = i64::min_value() + 1;
const MAX_INFINITY: i64 = i64::max_value();

macro_rules! get_opp {
    ($e:expr) => {
        match $e {
            Some(a) => Some(!a),
            _ => unreachable!(),
        }
    };
}

// negamax_try
fn ab_negamax(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    table: &[[[u64; 2]; SIZE_BOARD]; SIZE_BOARD],
    zhash: &mut u64,
    tt: &mut Vec<zobrist::TT>,
    current_depth: &mut i8,
    actual: Option<bool>,
    actual_catch: &mut isize,
    opp_catch: &mut isize,
    last_move: Option<(usize, usize)>,
    alpha: &mut i64,
    beta: &mut i64,
    color: &mut i8,
) -> (i64, Option<(usize, usize)>) {
    let mut tte = zobrist::retrieve_tt_from_hash(tt, zhash);
    let alpha_orig = *alpha;

    if tte.is_valid && tte.depth == DEPTH_MAX - *current_depth {
        if tte.r#type == zobrist::TypeOfEl::Exact {
            return (tte.value, tte.r#move);
        } else if tte.r#type == zobrist::TypeOfEl::Lowerbound  {
            *alpha = i64::max(*alpha, tte.value);
        } else if tte.r#type == zobrist::TypeOfEl::Upperbound {
             *beta = i64::min(*beta, tte.value);
        }

        if *alpha >= *beta {
            return (tte.value, tte.r#move);
        }
    }

    if *current_depth == DEPTH_MAX || board_state_win(board, actual_catch, opp_catch) {
        let weight = heuristic::first_heuristic_hint(
            board,
            actual,
            actual_catch,
            opp_catch,
            &mut (DEPTH_MAX - *current_depth),
        );
        return (weight, None);
    }

    // Otherwise bubble up values from below
    let mut best_move: Option<(usize, usize)> = None;
    let mut best_score = MIN_INFINITY;

    // Collect moves
    let available_positions = get_space(board, actual, *actual_catch);

    for (line, col, _) in available_positions {
        let removed = change_board(board, line, col, actual, table, zhash);
        *actual_catch += removed.len() as isize;

        // Recurse
        let (recursed_score, _) = ab_negamax(
            board,
            table,
            zhash,
            tt,
            &mut (*current_depth + 1),
            get_opp!(actual),
            opp_catch,
            actual_catch,
            Some((line, col)),
            &mut (-*beta),
            &mut (-*alpha),
            &mut (-*color),
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
        remove_last_pawn(board, line, col, actual, removed, table, zhash);

        if *alpha >= *beta {
            best_score = *alpha;
            best_move = Some((line, col));
            break ;
            // return (*alpha, best_move);
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
     tte.depth = *current_depth;
     zobrist::store_tt_entry(tt, zhash, tte);

    (best_score, best_move)
}

fn get_best_move(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    table: &[[[u64; 2]; SIZE_BOARD]; SIZE_BOARD],
    zhash: &mut u64,
    tt: &mut Vec<zobrist::TT>,
    actual: Option<bool>,
    actual_catch: &mut isize,
    opp_catch: &mut isize,
    last_move: Option<(usize, usize)>,
    alpha: &mut i64,
    beta: &mut i64,
) -> (usize, usize) {
    let (_, r#move): (i64, Option<(usize, usize)>) = ab_negamax(
        board,
        table,
        zhash,
        tt,
        &mut 0,
        actual,
        actual_catch,
        opp_catch,
        last_move,
        alpha,
        beta,
        &mut 1,
    );
    match r#move {
        Some(x) => x,
        _ => {
            sleep(Duration::new(10, 0));
            unreachable!();
        }
    }
}

fn ia(
    game: &mut game::Game,
    (table, mut hash): (&[[[u64; 2]; SIZE_BOARD]; SIZE_BOARD], u64),
) -> (usize, usize) {
    let mut player_catch = game.get_actual_player().nb_of_catch;
    let mut opponent_catch = game.get_opponent().nb_of_catch;
    let mut board = game.board;
    let pawn = game.player_to_pawn();
    let mut tt = zobrist::initialize_transposition_table();

    get_best_move(
        &mut board,
        table,
        &mut hash,
        &mut tt,
        pawn,
        &mut player_catch,
        &mut opponent_catch,
        None,
        &mut MIN_INFINITY,
        &mut MAX_INFINITY,
    )
}

pub fn get_ia(game: &mut game::Game, ztable: &[[[u64; 2]; SIZE_BOARD]; SIZE_BOARD]) -> (usize, usize) {
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
                game::TypeOfParty::Standard => ia(game, (ztable, hash)),
            }
        }
        _ => {
            let ret = ia(game, (ztable, hash));
            ret
        }
    }
}
