// use super::super::model::point;
// use rand::distributions::{Distribution, Uniform};
// use rand::thread_rng;
use super::super::checks::capture;
use super::super::checks::search_space;
use super::super::model::game;
// use super::super::model::player;
use rand::seq::SliceRandom;
use super::zobrist;
use super::zobrist::Move;
use super::zobrist::TypeOfEl;
// use super::super::player;

fn evaluate() -> i32 {
    10
}

// alpha beta memory
fn alpha_beta_w_memory(
    game: &mut game::Game,
    table: &[[[u64; 2]; 19]; 19],
    zhash: &mut u64,
    tt: &mut Vec<zobrist::TT>,
    depth: &mut i8,
    alpha: &mut i32,
    beta: &mut i32
) -> (i32,Option<(usize, usize)>) {
    let mut value: i32;
    let mut best_value: i32;
    let mut best_mov = Move::Leaf;
    let tte = zobrist::retrieve_tt_from_hash(tt, zhash);

    // If I can retrieve interesting data from TT
    // On testera avec ==
    if tte.r#type != zobrist::TypeOfEl::Empty && tte.depth >= *depth {
        if tte.r#type == zobrist::TypeOfEl::Exact {
            let mov = tte.r#move.unwrap_unsafe();
            return (tte.value,Some(mov));
        }

        if tte.r#type == zobrist::TypeOfEl::Lowerbound && tte.value > *alpha {
            *alpha = tte.value; // update lowerbound value (alpha)
        } else if tte.r#type == zobrist::TypeOfEl::Upperbound && tte.value < *beta {
            *beta = tte.value; // update upperbound value (beta)
        }

        if *alpha >= *beta {
            // match tte.r#move {
                let mov2 = tte.r#move.unwrap_unsafe();
                return (tte.value,Some(mov2));
                // Move::Some((i,j)) => return (tte.value,Some((i,j))),
                // // Move::Leaf => return (tte.value,None),
                // _=> unreachable!(),
            // } // Directly cut branch
        }
    }

    // Process Leaf or end of game
    if *depth == 0 || game.check_win() {
        // value = evaluate(board);
        // Line below --> debug
        value = evaluate();
        // Stocke-t-on ou non ici ??
        // if value <= *alpha { // a lowerbound value
        //     zobrist::store_tt_entry(tt, zhash, &value, TypeOfEl::Lowerbound, depth, Move::Leaf);
        // } else if value >= *beta { // an upperbound value
        //     zobrist::store_tt_entry(tt, zhash, &value, TypeOfEl::Upperbound, depth, Move::Leaf);
        // } else { // a true minimax value
        //     zobrist::store_tt_entry(tt, zhash, &value, TypeOfEl::Exact, depth, Move::Leaf);
        // }
        return (value, None);
    }
    
    // First check already known move (reordering)
    if tte.r#type != zobrist::TypeOfEl::Empty {
        // Place pawn
        match tte.r#move {
            Move::Some((i,j)) => game.ia_change_board_from_input(i, j, &table, zhash),
            _ => unreachable!(),
        }
        // Collect value of this branch
        let (tmp_best,_) = alpha_beta_w_memory(game, table, zhash, tt, &mut(*depth-1),&mut(-*beta),&mut(-*alpha));
        best_value = -tmp_best;
        // Remove pawn
        game.ia_clear_last_move(table, zhash);
        best_mov = tte.r#move;
    } else {
        best_value = i32::min_value() + 1;   // ????? DANGEROUS CAST ?????
    }

    if best_value < *beta {
        //
        let available_positions = search_space::search_space(game);
        for i in 0..available_positions.len() {
            if Move::Some(available_positions[i]) != tte.r#move {
                // println!("zhash_before-change: {}| depth: {}", zhash, depth);
                game.ia_change_board_from_input(available_positions[i].0, available_positions[i].1, &table, zhash);
                // println!("zhash_after-change: {}| depth: {}", zhash, depth);
                let (val,_) = alpha_beta_w_memory(game, table, zhash, tt, &mut(*depth-1),&mut(-*beta),&mut(-*alpha));
                value = -val;
                game.ia_clear_last_move(table, zhash);
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

    if best_value <= *alpha { // a lowerbound value
        zobrist::store_tt_entry(tt, zhash, &best_value, TypeOfEl::Lowerbound, depth, best_mov); 
    } else if best_value >= *beta {  // an upperbound value
        zobrist::store_tt_entry(tt, zhash, &best_value, TypeOfEl::Upperbound, depth, best_mov);
    } else {  // a true minimax value
        zobrist::store_tt_entry(tt, zhash, &best_value, TypeOfEl::Exact, depth, best_mov);
    }
    match best_mov {
        Move::Some((i,j)) => return (best_value,Some((i,j))),
        Move::Leaf => return (best_value,None), // Full board case
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
fn ia(
    game: &mut game::Game,
    (table, mut hash):([[[u64; 2]; 19]; 19], u64)
) -> (usize, usize) {
    let mut  _player = game.get_actual_player();
    let mut _oppenent = game.get_opponent();
    // let mut best_position: (usize, usize) = (0,0);
    // let mut best_score = 0;
    let mut tt = zobrist::initialize_transposition_table();
    // let available_positions = search_space::search_space(game);
    
    // available_positions.iter().for_each(| &(i,j) | {
    //     // Place pawn on board && zobrit
    //     game.ia_change_board_from_input(i, j, &table, &mut hash);
    //     match mtdf(game, player, opponent, (&table, &mut hash), 9) {
    //         x if x < best_score => (),
    //         x if x >= best_score => { best_score = x ; best_position = (i,j) },
    //     }
    //     // Remove pawn on board && zobrit
    //     game.ia_clear_last_move(&table, &mut hash);
    // }); 
    match alpha_beta_w_memory(game, &table, &mut hash, &mut tt, &mut 5,
        &mut (i32::min_value() + 1), &mut (i32::max_value())
    ) {
        (_, Some(best_position)) => best_position,
        (_, None) => unreachable!(),
    }

}

// Need to take history into account, found some issue with double_three
pub fn get_ia(game: &mut game::Game) -> (usize,usize) {
    // Initialize Zobrit hash
    let (table, hash): ([[[u64; 2]; 19]; 19], u64) = zobrist::board_to_zhash(&game.board);
    let mut rng = rand::thread_rng();

    match game.history.len() {
        0 => (9,9),
        2 => {
            // println!("{}", "passé dans 1");
            let (dir_line, dir_col) = capture::DIRS.choose(&mut rng).expect("Error in random extraction");
            match game.type_of_party {
                game::TypeOfParty::Pro      => ((9 + dir_line * 3) as usize, (9 + dir_col * 3) as usize),
                game::TypeOfParty::Longpro  => ((9 + dir_line * 4) as usize, (9 + dir_col * 4) as usize),
                game::TypeOfParty::Standard => ia(game, (table, hash)),
            }
        },
        _ => ia(game, (table, hash)),
    }
}
