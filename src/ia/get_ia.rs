// use super::super::model::point;
// use rand::distributions::{Distribution, Uniform};
// use rand::thread_rng;
use super::super::checks::capture;
use super::super::checks::search_space;
use super::super::model::game;
// use super::super::model::player;
use super::heuristic;
use super::zobrist;
use super::zobrist::Move;
use super::zobrist::TypeOfEl;
use rand::seq::SliceRandom;
// use super::super::player;

macro_rules! string_of_index {
    ($line:expr, $col:expr) => {{
        let col: char = std::char::from_u32('A' as u32 + *$col as u32)
            .expect("Could not convert number to char");
        let line = *$line;
        format!("{}{}", col, line)
    }};
}
const DEPTH_MAX: i8 = 3;

// alpha beta memory
fn alpha_beta_w_memory(
    game: &mut game::Game,
    table: &[[[u64; 2]; 19]; 19],
    zhash: &mut u64,
    tt: &mut Vec<zobrist::TT>,
    depth: &mut i8,
    alpha: &mut i64,
    beta: &mut i64,
) -> (i64, Option<(usize, usize)>) {
    let mut value: i64;
    let mut best_value: i64;
    let mut best_mov = Move::Unitialized;
    let tte = zobrist::retrieve_tt_from_hash(tt, zhash);

    //    println!("call alphabeta prof {}", depth);
    // If I can retrieve interesting data from TT
    // On testera avec ==
    if tte.r#type != zobrist::TypeOfEl::Empty && tte.depth >= *depth {
        if tte.r#type == zobrist::TypeOfEl::Exact {
            match tte.r#move {
                // let mov2 = tte.r#move.unwrap_unsafe();
                // return (tte.value,Some(mov2));
                Move::Some((i, j)) => return (tte.value, Some((i, j))),
                _ => unreachable!(),
            }
        }

        if tte.r#type == zobrist::TypeOfEl::Lowerbound && tte.value > *alpha {
            *alpha = tte.value; // update lowerbound value (alpha)
        } else if tte.r#type == zobrist::TypeOfEl::Upperbound && tte.value < *beta {
            *beta = tte.value; // update upperbound value (beta)
        }

        if *alpha >= *beta {
            match tte.r#move {
                // let mov2 = tte.r#move.unwrap_unsafe();
                // return (tte.value,Some(mov2));
                Move::Some((i, j)) => return (tte.value, Some((i, j))),
                _ => unreachable!(),
            } // Directly cut branch
        }
    }

    // Process Leaf or end of game
    if *depth == 0 || game.check_win_hint() {
        // value = evaluate(board);
        // Line below --> debug

        value = heuristic::first_heuristic(
            game.board,
            game.get_actual_player(),
            game.get_opponent(),
            depth,
        );
        //println!("value : {}", value);
        // Stocke-t-on ou non ici ??
        if value <= *alpha {
            // a lowerbound value
            zobrist::store_tt_entry(tt, zhash, &value, TypeOfEl::Lowerbound, depth, Move::Some(game.history[game.history.len()-1]));
        } else if value >= *beta {
            // an upperbound value
            zobrist::store_tt_entry(tt, zhash, &value, TypeOfEl::Upperbound, depth, Move::Some(game.history[game.history.len()-1]));
        } else {
            // a true minimax value
            zobrist::store_tt_entry(tt, zhash, &value, TypeOfEl::Exact, depth, Move::Some(game.history[game.history.len()-1]));
        }
        return (value, Some(game.history[game.history.len()-1]));
    }

    // First check already known move (reordering)
    if tte.r#type != zobrist::TypeOfEl::Empty && tte.r#move != zobrist::Move::Unitialized {
        // Place pawn
        match tte.r#move {
            Move::Some((i, j)) => {
                if game.board[i][j] == None {
                    println!("Nullos");
                    game.ia_change_board_from_input_hint(i, j, &table, zhash);
                    // Collect value of this branch
                    let (tmp_best, _) = alpha_beta_w_memory(
                        game,
                        table,
                        zhash,
                        tt,
                        &mut (*depth - 1),
                        &mut (-*beta),
                        &mut (-*alpha),
                    );
                    best_value = -tmp_best;
                    // Remove pawn
                    game.ia_clear_last_move_hint(table, zhash);
                    best_mov = tte.r#move;
                } else {
                    best_value = i64::min_value() + 1;
                }
            },
            _ => unreachable!(),
        }
        
    } else {
        best_value = i64::min_value() + 1; // ????? DANGEROUS CAST ?????
    }

    if best_value < *beta {
        //
        let available_positions = search_space::search_space(game);
        for i in 0..available_positions.len() {
            if Move::Some(available_positions[i]) != tte.r#move {
                // println!("zhash_before-change: {}| depth: {}", zhash, depth);
                if game.board[available_positions[i].0][available_positions[i].1] != None {
                    continue;
                    game.history
                        .iter()
                        .for_each(|(x, y)| print!("{}//", string_of_index!(x, y)));
                    println!("-------------");
                    game.history_capture
                        .iter()
                        .for_each(|(_, ((x1, y1), (x2, y2)))| {
                            print!(
                                "{}//{}---",
                                string_of_index!(x1, y1),
                                string_of_index!(x2, y2)
                            )
                        });
                    println!("-------------");
                    available_positions
                        .iter()
                        .for_each(|(x, y)| print!("{}//", string_of_index!(x, y)));
                    println!("-------------");
                    println!(
                        "{}",
                        string_of_index!(&available_positions[i].0, &available_positions[i].1)
                    );
                    println!("Nullos1");
                }
                //                println!(
                //                    "{}//{}:{}",
                //                    i, available_positions[i].0, available_positions[i].1
                //                );
                game.ia_change_board_from_input_hint(
                    available_positions[i].0,
                    available_positions[i].1,
                    &table,
                    zhash,
                );
                // println!("zhash_after-change: {}| depth: {}", zhash, depth);
                let (val, _) = alpha_beta_w_memory(
                    game,
                    table,
                    zhash,
                    tt,
                    &mut (*depth - 1),
                    &mut (-*beta),
                    &mut (-*alpha),
                );
                value = -val;
                game.ia_clear_last_move_hint(table, zhash);
                // println!("zhash_after-recursive: {}| depth: {}", zhash, depth);
                if value > best_value {
                    best_value = value;
                    best_mov = Move::Some(available_positions[i]);
                }
                if best_value > *alpha {
                    *alpha = best_value;
                }
                if best_value >= *beta {
                    break;
                }
            }
        }
    }

    if best_value <= *alpha {
        // a lowerbound value
        zobrist::store_tt_entry(
            tt,
            zhash,
            &best_value,
            TypeOfEl::Lowerbound,
            depth,
            best_mov,
        );
    } else if best_value >= *beta {
        // an upperbound value
        zobrist::store_tt_entry(
            tt,
            zhash,
            &best_value,
            TypeOfEl::Upperbound,
            depth,
            best_mov,
        );
    } else {
        // a true minimax value
        zobrist::store_tt_entry(tt, zhash, &best_value, TypeOfEl::Exact, depth, best_mov);
    }
    match best_mov {
        Move::Some((i, j)) => return (best_value, Some((i, j))),
        Move::Unitialized => unreachable!(),
    }
    // (best_value, best_mov)
}

// function mtdf(root, f, d) is
//     g := f
//     upperBound := +∞
//     lowerBound := −∞

//     while lowerBound < upperBound do
//         β := max(g, lowerBound + 1)
//         g := AlphaBetaWithMemory(root, β − 1, β, d)
//         if g < β then
//             upperBound := g
//         else
//             lowerBound := g

//     return g

// Aim of the function :
// Heart of AI, parse all available position close to a piece
// and apply the mtd-f algorithm on it with depth of 10
fn ia(game: &mut game::Game, (table, mut hash): ([[[u64; 2]; 19]; 19], u64)) -> (usize, usize) {
    let mut _player = game.get_actual_player();
    let mut _oppenent = game.get_opponent();
    // let mut best_position: (usize, usize) = (0,0);
    // let mut best_score = 0;
    let mut depth_max = DEPTH_MAX;
    let mut tt = zobrist::initialize_transposition_table();
    // let available_positions = search_space::search_space(game);

    // available_positions.iter().for_each(| &(i,j) | {
    //     // Place pawn on board && zobrit
    //     game.ia_change_board_from_input_hint(i, j, &table, &mut hash);
    //     match mtdf(game, player, opponent, (&table, &mut hash), 9) {
    //         x if x < best_score => (),
    //         x if x >= best_score => { best_score = x ; best_position = (i,j) },
    //     }
    //     // Remove pawn on board && zobrit
    //     game.ia_clear_last_move(&table, &mut hash);
    // });
    match alpha_beta_w_memory(
        game,
        &table,
        &mut hash,
        &mut tt,
        &mut depth_max,
        &mut (i64::min_value() + 1),
        &mut (i64::max_value()),
    ) {
        (_, Some(best_position)) => best_position,
        (_, None) => unreachable!(),
    }
}

// Need to take history into account, found some issue with double_three
pub fn get_ia(game: &mut game::Game) -> (usize, usize) {
    // Initialize Zobrit hash
    let (table, hash): ([[[u64; 2]; 19]; 19], u64) = zobrist::board_to_zhash(&game.board);
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
            ret
        }
    }
}
