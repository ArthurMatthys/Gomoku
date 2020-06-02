// use super::super::model::point;
// use rand::distributions::{Distribution, Uniform};
// use rand::thread_rng;
use super::super::checks::capture;
use super::super::checks::double_three;
use super::super::checks::search_space;
use super::super::model::game;
use super::super::model::player;
use super::super::render::board::SIZE_BOARD;
use std::time::Duration;
// use super::super::model::player;
use super::heuristic;
use super::zobrist;
use rand::seq::SliceRandom;
// use super::super::player;

const DEPTH_MAX: i8 = 5;
const MIN_INFINITY: i64 = i64::min_value() + 1;
const MAX_INFINITY: i64 = i64::max_value();

macro_rules! string_of_index {
    ($line:expr, $col:expr) => {{
        let col: char = std::char::from_u32('A' as u32 + *$col as u32)
            .expect("Could not convert number to char");
        let line = *$line;
        format!("{}{}", col, line)
    }};
}

macro_rules! valid_pos {
    ($x: expr, $y: expr) => {
        $x > 0 && $x < SIZE_BOARD as isize && $y > 0 && $y < SIZE_BOARD as isize
    };
}

macro_rules! get_space {
    ($board:expr, $actual_player:expr) => {{
        let mut ret = vec![];
        for x in 0..19 {
            for y in 0..19 {
                let value = $board[x][y];
                if value == None {
                    for &(dx, dy) in capture::DIRS.iter() {
                        let new_x = x as isize + dx;
                        let new_y = y as isize + dy;
                        if valid_pos!(new_x, new_y)
                            && $board[new_x as usize][new_y as usize] != None
                        {
                            if !double_three::check_double_three_hint(
                                $board,
                                $actual_player,
                                //get_opp!($actual_player),
                                x as isize,
                                y as isize,
                            ) {
                                ret.push((x, y));
                            }
                        }
                    }
                }
            }
        }
        ret
    }};
}

macro_rules! get_zindex_from_pawn {
    ($e:expr) => {
        match $e {
            Some(true) => 1,
            Some(false) => 0,
            _ => unreachable!(),
        }
    };
}

macro_rules! get_opp {
    ($e:expr) => {
        match $e {
            Some(a) => Some(!a),
            _ => unreachable!(),
        }
    };
}

macro_rules! add_zhash {
    ($table:expr, $zhash:expr, $x:expr, $y:expr, $piece:expr) => {
        *$zhash ^= $table[$x as usize][$y as usize][zobrist::ZPIECES[$piece]];
    };
}

macro_rules! winner_move {
    ($board:expr, $last_move:expr) => {{
        if let Some((x, y)) = $last_move {
            let ways = [-1, 1];
            let pawn = $board[x][y];
            let res = capture::DIRS
                .iter()
                .map(|&(dx, dy)| {
                    let mut count = 1;
                    let mut new_x = x as isize;
                    let mut new_y = y as isize;
                    ways.iter().for_each(|&way| loop {
                        new_x += dx * way;
                        new_y += dy * way;
                        if valid_pos!(new_x, new_y)
                            && $board[new_x as usize][new_y as usize] == pawn
                        {
                            count += 1;
                        } else {
                            break;
                        }
                    });
                    count >= 5
                })
                .collect::<Vec<bool>>();
            res.iter().any(|&x| x)
        } else {
            false
        }
    }};
}

fn remove_last_pawn(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    x: usize,
    y: usize,
    pawn: Option<bool>,
    removed: Vec<((isize, isize), (isize, isize))>,
    table: &[[[u64; 2]; SIZE_BOARD]; SIZE_BOARD],
    zhash: &mut u64,
) {
    let old = get_opp!(pawn);
    board[x][y] = None;
    add_zhash!(table, zhash, x, y, get_zindex_from_pawn!(pawn));
    removed.iter().for_each(|&((x1, y1), (x2, y2))| {
        add_zhash!(table, zhash, x1, y1, get_zindex_from_pawn!(old));
        board[x1 as usize][y1 as usize] = old;
        add_zhash!(table, zhash, x2, y2, get_zindex_from_pawn!(old));
        board[x2 as usize][y2 as usize] = old;
    })
}

fn change_board(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    x: usize,
    y: usize,
    pawn: Option<bool>,
    table: &[[[u64; 2]; SIZE_BOARD]; SIZE_BOARD],
    zhash: &mut u64,
) -> Vec<((isize, isize), (isize, isize))> {
    let mut removed = vec![];
    board[x][y] = pawn;
    add_zhash!(table, zhash, x, y, get_zindex_from_pawn!(pawn));
    let opp = get_opp!(pawn);
    for &(dx, dy) in capture::DIRS.iter() {
        let mut count = 0;
        let mut new_x = x as isize;
        let mut new_y = y as isize;
        for _ in 0..3 {
            new_x += dx;
            new_y += dy;
            if !valid_pos!(new_x, new_y) {
                count = 0;
                break;
            } else if board[new_x as usize][new_y as usize] != opp {
                break;
            } else if board[new_x as usize][new_y as usize] == opp {
                count += 1;
            }
        }
        if count == 2 && board[new_x as usize][new_y as usize] == pawn {
            let (x1, y1) = (new_x - dx, new_y - dy);
            let (x2, y2) = (x1 - dx, y1 - dy);
            board[x1 as usize][y1 as usize] = None;
            add_zhash!(table, zhash, x1, y1, get_zindex_from_pawn!(opp));
            board[x2 as usize][y2 as usize] = None;
            add_zhash!(table, zhash, new_x, new_y, get_zindex_from_pawn!(opp));
            removed.push(((x1, y1), (x2, y2)));
        }
    }

    removed
}

// negamax_try
fn ab_negamax_tt(
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
) -> (i64, Option<(usize, usize)>) {
    
    let alpha_orig = *alpha;
    let mut tte = zobrist::retrieve_tt_from_hash(tt, zhash);
    
    if tte.is_valid && tte.depth >= DEPTH_MAX - *current_depth {
        if tte.r#type == zobrist::TypeOfEl::Exact {
            return (tte.value, tte.r#move);
        }
        else if tte.r#type == zobrist::TypeOfEl::Lowerbound  {
            *alpha = i64::max(*alpha, tte.value);
        } else if tte.r#type == zobrist::TypeOfEl::Upperbound {
             *beta = i64::min(*beta, tte.value);
        }

        if *alpha >= *beta {
            return (tte.value, tte.r#move);
        }
    }

    // End game
    if *current_depth == DEPTH_MAX || *actual_catch >= 5 || winner_move!(board, last_move) {
        // let lol = heuristic::first_heuristic_hint(board, actual, actual_catch, opp_catch, &mut (DEPTH_MAX - *current_depth));
        let lol = -10;
        println!("evaluation - first print | catch:{} | depth: {}| heur: {}", actual_catch, current_depth, lol);
        for i in 0..19 {
            for j in 0..19 {
                match board[j][i] {
                    Some(true) => print!("⊖"),
                    Some(false) => print!("⊕"),
                    None => print!("_"),
                }
            }
            println!();
        }
        // in recurse
        // println!("leaf/winning, depth:{}", *current_depth);
        return (lol, None);
    }

    // Otherwise bubble up values from below
    let mut best_move: Option<(usize, usize)> = None;
    let mut best_score = MIN_INFINITY;
    
    // Collect moves
    let available_positions = get_space!(board, actual);
    // let available_positions2 = get_space!(board, actual);

    // Go through each move
    for (line, col) in available_positions {
        // // debug
        // if board[line][col] != None {
        //     unreachable!();
        // }
        println!("--------------------------");
        println!("board - first print | catch:{} | depth: {}", actual_catch, current_depth);
        for i in 0..19 {
            for j in 0..19 {
                match board[j][i] {
                    Some(true) => print!("⊖"),
                    Some(false) => print!("⊕"),
                    None => print!("_"),
                }
            }
            println!();
        }

        // let mut board_cloned = board.clone();
        // let mut catch = actual_catch.clone();
        // println!("board_copy - before change | catch:{}", catch);
        // for i in 0..19 {
        //     for j in 0..19 {
        //         match board_cloned[j][i] {
        //             Some(true) => print!("⊖"),
        //             Some(false) => print!("⊕"),
        //             None => print!("_"),
        //         }
        //     }
        //     println!();
        // }

        let removed = change_board(board, line, col, actual, table, zhash);
        *actual_catch += removed.len() as isize;

        println!("board - after change | catch:{} | depth: {}", *actual_catch, current_depth);
        for i in 0..19 {
            for j in 0..19 {
                match board[j][i] {
                    Some(true) => print!("⊖"),
                    Some(false) => print!("⊕"),
                    None => print!("_"),
                }
            }
            println!();
        }

        // Recurse
        let (recursed_score,_) = ab_negamax_tt(board,
                                            table,
                                            zhash,
                                            tt,
                                            &mut (*current_depth + 1),
                                            get_opp!(actual),
                                            opp_catch,
                                            actual_catch,
                                            Some((line,col)),
                                            &mut (-*beta),
                                            &mut (-i64::max(*alpha, best_score)));
        
        let current_score = -recursed_score;

        *actual_catch -= removed.len() as isize;
        remove_last_pawn(board, line, col, actual, removed, table, zhash);

        println!("board - after repair | catch:{} | depth: {}", *actual_catch, current_depth);
        for i in 0..19 {
            for j in 0..19 {
                match board[j][i] {
                    Some(true) => print!("⊖"),
                    Some(false) => print!("⊕"),
                    None => print!("_"),
                }
            }
            println!();
        }

        // println!("debug: {}|{}|{}", *current_depth, current_score, best_score);
        // Update the best score
        if current_score > best_score {
            // println!("update_score, depth:{}", *current_depth);
            best_score = current_score;
            best_move = Some((line, col));
    
            // If we’re outside the bounds, then prune: exit immediately
            if best_score >= *beta {
                break ;
                // println!("prune, depth:{}", *current_depth);
                // return (best_score, best_move);
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
     tte.depth = *current_depth;
     zobrist::store_tt_entry(tt, zhash, tte);
    
    (best_score, best_move)
}

fn  get_best_move(
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
    let (_, r#move): (i64,Option<(usize, usize)>) = ab_negamax_tt(
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
                                                );
    match r#move {
        Some(x) => x,
        _ => unreachable!(),
    }
}

fn ia(
    game: &mut game::Game,
    (table, mut hash): ([[[u64; 2]; SIZE_BOARD]; SIZE_BOARD], u64),
) -> (usize, usize) {
    // let player = game.get_actual_player();
    let mut player_catch = game.get_actual_player().nb_of_catch;
    let mut opponent_catch = game.get_opponent().nb_of_catch;
    let mut board = game.board;
    let pawn = game.player_to_pawn();
    
    // let mut depth_max = DEPTH_MAX;
    let mut tt = zobrist::initialize_transposition_table();

    get_best_move(
        &mut board,
        &table,
        &mut hash,
        &mut tt,
        pawn,
        &mut player_catch,
        &mut opponent_catch,
        None,
        &mut MIN_INFINITY,
        &mut MAX_INFINITY,
    )
    
    // match alpha_beta_w_memory_hint(
    //     &mut board,
    //     player.bool_type,
    //     &mut player_catch,
    //     &mut opponent_catch,
    //     None,
    //     &table,
    //     &mut hash,
    //     &mut tt,
    //     &mut depth_max,
    //     &mut (i64::min_value() + 1),
    //     &mut (i64::max_value()),
    // ) {        
    //     (_, Some(best_position)) => best_position,
    //     (_, None) => unreachable!(),
    // }
}

// Need to take history into account, found some issue with double_three
pub fn get_ia(game: &mut game::Game) -> (usize, usize) {
    // Initialize Zobrit hash
    let (table, hash): ([[[u64; 2]; SIZE_BOARD]; SIZE_BOARD], u64) =
        zobrist::board_to_zhash(&game.board);
    let mut rng = rand::thread_rng();

    match game.history.len() {
        0 => (9, 9),
        2 => {
            // println!("{}", "passé dans 1");
            let (dir_line, dir_col) = capture::DIRS
                .choose(&mut rng)
                .expect("Error in random extraction");
            match game.type_of_party {
                game::TypeOfParty::Pro => ((9 + dir_line * 3) as usize, (9 + dir_col * 3) as usize),
                game::TypeOfParty::Longpro => {
                    ((9 + dir_line * 4) as usize, (9 + dir_col * 4) as usize)
                }
                game::TypeOfParty::Standard => ia(game, (table, hash)),
            }
        }
        _ => {
            let ret = ia(game, (table, hash));
            println!("move found");
            ret
        }
    }
}
