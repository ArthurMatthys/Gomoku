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
use super::zobrist::Move;
use super::zobrist::TypeOfEl;
use rand::seq::SliceRandom;
// use super::super::player;

const DEPTH_MAX: i8 = 4;
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

// fn alpha_beta_w_memory_hint(
//     board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
//     actual: Option<bool>,
//     actual_catch: &mut isize,
//     opp_catch: &mut isize,
//     last_move: Option<(usize, usize)>,
//     table: &[[[u64; 2]; SIZE_BOARD]; SIZE_BOARD],
//     zhash: &mut u64,
//     tt: &mut Vec<zobrist::TT>,
//     depth: &mut i8,
//     alpha: &mut i64,
//     beta: &mut i64,
// ) -> (i64, Option<(usize, usize)>) {
//     let mut value: i64 = i64::min_value();
//     let mut best_value: i64;
//     let mut best_mov = Move::Unitialized;
//     let tte = zobrist::retrieve_tt_from_hash(tt, zhash);

//     //    println!("call alphabeta prof {}", depth);
//     // If I can retrieve interesting data from TT
//     // On testera avec ==
//     if tte.r#type != zobrist::TypeOfEl::Empty && tte.depth >= *depth {
//         if tte.r#type == zobrist::TypeOfEl::Exact {
//             match tte.r#move {
//                 // let mov2 = tte.r#move.unwrap_unsafe();
//                 // return (tte.value,Some(mov2));
//                 Move::Some((i, j)) => return (tte.value, Some((i, j))),
//                 _ => unreachable!(),
//             }
//         }

//         if tte.r#type == zobrist::TypeOfEl::Lowerbound && tte.value > *alpha {
//             *alpha = i64::max(*alpha, tte.value); // update lowerbound value (alpha)
//         } else if tte.r#type == zobrist::TypeOfEl::Upperbound && tte.value < *beta {
//             *beta = i64::min(*beta, tte.value); // update upperbound value (beta)
//         }

//         if *alpha >= *beta {
//             match tte.r#move {
//                 // let mov2 = tte.r#move.unwrap_unsafe();
//                 // return (tte.value,Some(mov2));
//                 Move::Some((i, j)) => return (tte.value, Some((i, j))),
//                 _ => unreachable!(),
//             } // Directly cut branch
//         }
//     }

//     // Process Leaf or end of game
//     if *depth == 0 || winner_move!(board, last_move) || *actual_catch >= 5 || *opp_catch >= 5 {
//         // value = evaluate(board);
//         // Line below --> debug

//         value = -heuristic::first_heuristic_hint(board, actual, actual_catch, opp_catch, depth);
//         return (value, last_move);
//     }

//     // First check already known move (reordering)
//     if tte.r#type != zobrist::TypeOfEl::Empty && tte.r#move != zobrist::Move::Unitialized {
//         // Place pawn
//         match tte.r#move {
//             Move::Some((i, j)) => {
//                 //add TODO (modify zhash)
//                 let removed = change_board(board, i, j, actual, table, zhash);
//                 *actual_catch += removed.len() as isize;
//                 //game.ia_change_board_from_input_hint(i, j, &table, zhash);
//                 // Collect value of this branch
//                 let (tmp_best, _) = alpha_beta_w_memory_hint(
//                     board,
//                     get_opp!(actual),
//                     opp_catch,
//                     actual_catch,
//                     Some((i, j)),
//                     table,
//                     zhash,
//                     tt,
//                     &mut (*depth - 1),
//                     &mut (-*beta),
//                     &mut (-*alpha),
//                 );
//                 best_value = -tmp_best;
//                 *actual_catch -= removed.len() as isize;
//                 // Remove pawn TODO DONE
//                 remove_last_pawn(board, i, j, actual, removed, table, zhash);
//                 //game.ia_clear_last_move_hint(table, zhash);
//                 best_mov = tte.r#move;
//             }
//             _ => unreachable!(),
//         }
//     } else {
//         best_value = i64::min_value() + 1; // ????? DANGEROUS CAST ?????
//     }

//     if best_value < *beta {
//         // TODO search_space
//         //let available_positions = search_space::search_space(game);
//         let available_positions = get_space!(board, actual);
//         for i in 0..available_positions.len() {
//             if Move::Some(available_positions[i]) != tte.r#move {
//                 let (new_x, new_y) = available_positions[i];
//                 // println!("zhash_before-change: {}| depth: {}", zhash, depth);
//                 if board[new_x][new_y] != None {
//                     unreachable!();
//                 }
//                 let removed = change_board(board, new_x, new_y, actual, table, zhash);
//                 *actual_catch += removed.len() as isize;
//                 //TODO
//                 //game.ia_change_board_from_input_hint(
//                 //    available_positions[i].0,
//                 //    available_positions[i].1,
//                 //    &table,
//                 //    zhash,
//                 //);
//                 // println!("zhash_after-change: {}| depth: {}", zhash, depth);
//                 let (val, _) = alpha_beta_w_memory_hint(
//                     board,
//                     get_opp!(actual),
//                     opp_catch,
//                     actual_catch,
//                     Some((new_x, new_y)),
//                     table,
//                     zhash,
//                     tt,
//                     &mut (*depth - 1),
//                     &mut (-*beta),
//                     &mut (-*alpha),
//                 );
//                 value = -val;
//                 *actual_catch -= removed.len() as isize;
//                 remove_last_pawn(board, new_x, new_y, actual, removed, table, zhash);
//                 //TODO DONE
//                 //game.ia_clear_last_move_hint(table, zhash);
//                 // println!("zhash_after-recursive: {}| depth: {}", zhash, depth);
//                 if value > best_value {
//                     best_value = value;
//                     best_mov = Move::Some((new_x, new_y));
//                 }
//                 // *alpha = value.max(*alpha);
//                 // if *alpha >= *beta {
//                 //     break;
//                 // }
//                                if best_value > *alpha {
//                                    *alpha = best_value;
//                                }
//                                if best_value >= *beta {
//                                    break;
//                                }
//             }
//         }
//     }

//     if best_value <= *alpha {
//         // a lowerbound value
//         zobrist::store_tt_entry(
//             tt,
//             zhash,
//             &best_value,
//             TypeOfEl::Upperbound,
//             depth,
//             best_mov,
//         );
//     } else if best_value >= *beta {
//         // an upperbound value
//         zobrist::store_tt_entry(
//             tt,
//             zhash,
//             &best_value,
//             TypeOfEl::Lowerbound,
//             depth,
//             best_mov,
//         );
//     } else {
//         // a true minimax value
//         zobrist::store_tt_entry(tt, zhash, &best_value, TypeOfEl::Exact, depth, best_mov);
//     }
//     match best_mov {
//         Move::Some((i, j)) => return (best_value, Some((i, j))),
//         Move::Unitialized => { println!("last_info b_val:{}|bet:{}|alph:{}|val:{}|dp:{}|{}", best_value, *beta, *alpha, value, depth, match best_mov {
//             Move::Unitialized => "uninit",
//             _ => "init",
//         }); unreachable!() },
//     }
//     // (best_value, best_mov)
// }

// negamax_try
fn ab_negamax(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    table: &[[[u64; 2]; SIZE_BOARD]; SIZE_BOARD],
    zhash: &mut u64,
    current_depth: &mut i8,
    actual: Option<bool>,
    actual_catch: &mut isize,
    opp_catch: &mut isize,
    last_move: Option<(usize, usize)>,
    alpha: &mut i64,
    beta: &mut i64,
    color: &mut i8
) -> (i64, Option<(usize, usize)>) {
    // println!("entry: {}", current_depth);
    if *current_depth == DEPTH_MAX || *actual_catch >= 5 || winner_move!(board, last_move) {
        let lol = heuristic::first_heuristic_hint(
            board,
            actual,
            actual_catch,
            opp_catch,
            &mut (DEPTH_MAX - *current_depth),
        );
        // let lol = 10;
        println!(
            "evaluation - first print | catch:{} | depth: {}| heur: {}",
            actual_catch, current_depth, (lol * (*color as i64))
        );
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
        //        println!("leaf/winning, depth:{}", *current_depth);
        //        return (
        //            heuristic::first_heuristic_hint(
        //                board,
        //                actual,
        //                actual_catch,
        //                opp_catch,
        //                &mut (DEPTH_MAX - *current_depth),
        //            ),
        //            None,
        //        );
        // println!("leaf/winning, depth:{}", *current_depth);
        return (lol * (*color as i64), None);
        // return (10, None);
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
        println!(
            "board - first print | catch:{} | depth: {}",
            actual_catch, current_depth
        );
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

        println!(
            "board - after change | catch:{} | depth: {}",
            *actual_catch, current_depth
        );
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
        let (recursed_score, _) = ab_negamax(
            board,
            table,
            zhash,
            &mut (*current_depth + 1),
            actual,
            actual_catch,
            opp_catch,
            Some((line, col)),
            &mut (-*beta),
            &mut (-*alpha),
            &mut (-*color)
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

        let test: i64 = -1;
        
        println!(
            "board - after repair | catch:{} | depth: {} | current_score: {}",
            *actual_catch, current_depth, recursed_score * test.pow((*current_depth) as u32)
        );
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
            // If we’re outside the bounds, then prune: exit immediately
            if *alpha >= *beta {
                // best_move = Some((line, col));
                return (*alpha, best_move);
                // println!("prune, depth:{}", *current_depth);
            //    break ;
            }

    }
    //    println!(
    //        "normal_end: {}|{}|{}",
    //        available_positions2.len(),
    //        *current_depth,
    //        match best_move {
    //            None => "None",
    //            Some(_) => "otra",
    //        }
    //    );
    // println!("normal_end: {}|{}|{}", available_positions2.len(), *current_depth, match best_move {
    //     None => "None",
    //     Some(_) => "otra",
    // });
    (best_score, best_move)
}

fn get_best_move(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    table: &[[[u64; 2]; SIZE_BOARD]; SIZE_BOARD],
    zhash: &mut u64,
    actual: Option<bool>,
    actual_catch: &mut isize,
    opp_catch: &mut isize,
    last_move: Option<(usize, usize)>,
    alpha: &mut i64,
    beta: &mut i64,
) -> (usize, usize) {
    println!("ENTRY RECURS");
    let (_, r#move): (i64, Option<(usize, usize)>) = ab_negamax(
        board,
        table,
        zhash,
        &mut 0,
        get_opp!(actual),
        opp_catch,
        actual_catch,
        last_move,
        alpha,
        beta,
        &mut 1,
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
    let player = game.get_actual_player();
    let mut player_catch = game.get_actual_player().nb_of_catch;
    let mut opponent_catch = game.get_opponent().nb_of_catch;
    let mut board = game.board;
    let pawn = game.player_to_pawn();

    // let mut depth_max = DEPTH_MAX;
    // let mut tt = zobrist::initialize_transposition_table();

    get_best_move(
        &mut board,
        &table,
        &mut hash,
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
            println!("board without move :");
            for i in 0..19 {
                for j in 0..19 {
                    match game.board[j][i] {
                        Some(true) => print!("⊖"),
                        Some(false) => print!("⊕"),
                        None => print!("_"),
                    }
                }
                println!();
            }
            ret
        }
    }
}
