// use super::super::model::point;
// use rand::distributions::{Distribution, Uniform};
// use rand::thread_rng;
use super::super::checks::capture;
use super::super::checks::double_three;
use super::super::model::game;
use super::super::render::board::SIZE_BOARD;
// use super::super::model::player;
use super::heuristic;
use super::zobrist;
use rand::seq::SliceRandom;
// use super::super::player;

const DEPTH_MAX: i8 = 2;
const MIN_INFINITY: i64 = i64::min_value() + 1;
const MAX_INFINITY: i64 = i64::max_value();

//macro_rules! string_of_index {
//    ($line:expr, $col:expr) => {{
//        let col: char = std::char::from_u32('A' as u32 + *$col as u32)
//            .expect("Could not convert number to char");
//        let line = *$line;
//        format!("{}{}", col, line)
//    }};
//}

macro_rules! valid_pos {
    ($x: expr, $y: expr) => {
        $x > 0 && $x < SIZE_BOARD as isize && $y > 0 && $y < SIZE_BOARD as isize
    };
}

macro_rules! get_dir {
    ($dir:expr) => {
        match $dir {
            (1, 1) => (0, 1),
            (-1, -1) => (0, 0),
            (1, 0) => (1, 1),
            (-1, 0) => (1, 0),
            (1, -1) => (2, 1),
            (-1, 1) => (2, 0),
            (0, 1) => (3, 1),
            (0, -1) => (3, 0),
            (_, _) => unreachable!(),
        }
    };
}

macro_rules! get_other_edge {
    ($tuple:expr, $dir:expr) => {
        match $dir {
            0 => $tuple.1,
            1 => $tuple.2,
            _ => unreachable!(),
        }
    };
}

const SCORE_ALIGN: i64 = 100;
const SCORE_TAKE: i64 = 100;

fn get_board_score(
    align: i8,
    same_pawn: bool,
    edge: Option<bool>,
    nb_take: isize,
    align_opp: i8,
    edge_opp: Option<bool>,
) -> i64 {
    match same_pawn {
        false => match align {
            2 => {
                //Can take
                if edge == Some(true) {
                    if nb_take == 4 {
                        heuristic::INSTANT_WIN
                    } else {
                        //TODO
                        SCORE_TAKE.pow((align as u32 + 2) / 2)
                    }
                //SO
                } else if edge == Some(false) {
                    SCORE_TAKE.pow(2)
                //Close
                } else {
                    SCORE_TAKE.pow(2) / 2
                }
            }
            len => {
                //Close
                if edge == Some(true) {
                    SCORE_ALIGN.pow(len as u32) * 2
                //SO
                } else if edge == Some(false) {
                    SCORE_ALIGN.pow(len as u32) / 2
                //Close
                } else {
                    SCORE_ALIGN.pow(len as u32)
                }
            }
        },
        true => {
            let tot_align = align + align_opp;
            if tot_align == 5 {
                return heuristic::INSTANT_WIN;
            }
            let edged = match edge {
                Some(true) => 1,
                Some(false) => 0,
                None => 3,
            } + match edge_opp {
                Some(true) => 1,
                Some(false) => 0,
                None => 3,
            };
            match edged {
                //open
                0 => SCORE_ALIGN.pow(tot_align as u32),
                //so
                1 => SCORE_ALIGN.pow(tot_align as u32) / 2,
                //close
                2 => SCORE_ALIGN.pow(tot_align as u32) / 8,
                //so with board
                3 => SCORE_ALIGN.pow(tot_align as u32) / 4,
                //close with board
                4 => SCORE_ALIGN.pow(tot_align as u32) / 16,
                //close *2
                _ => 0,
            }
        }
    }
}

fn get_space(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    actual_player: Option<bool>,
    actual_take: isize,
) -> Vec<(usize, usize, i64)> {
    let mut ret = vec![];
    let score_board: [[[(i8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD] =
        heuristic::evaluate_board(board);
    for x in 0..19 {
        for y in 0..19 {
            let value = board[x][y];
            if value == None {
                for &(dx, dy) in capture::DIRS.iter() {
                    let new_x = x as isize + dx;
                    let new_y = y as isize + dy;
                    let mut check = 0;
                    let mut score = 0i64;
                    if valid_pos!(new_x, new_y) && board[new_x as usize][new_y as usize] != None {
                        if check != 0
                            || !double_three::check_double_three_hint(
                                board,
                                actual_player,
                                //get_opp!($actual_player),
                                x as isize,
                                y as isize,
                            )
                        {
                            let mut edge_opp = None;
                            let mut opp_align = 0;
                            let opp_x = x as isize - dx;
                            let opp_y = y as isize - dx;
                            let (dir, way) = get_dir!((dx, dy));
                            if valid_pos!(opp_x, opp_y) {
                                if board[opp_x as usize][opp_y as usize] == value {
                                    let opp_tuple =
                                        score_board[opp_x as usize][opp_y as usize][dir];
                                    opp_align = opp_tuple.0;
                                    edge_opp = get_other_edge!(opp_tuple, (way + 1) % 2);
                                } else if board[opp_x as usize][opp_y as usize] == None {
                                    edge_opp = Some(false);
                                } else {
                                    edge_opp = Some(true);
                                }
                            }
                            check = 1;
                            let tuple_focused = score_board[new_x as usize][new_y as usize][dir];
                            score += get_board_score(
                                tuple_focused.0,
                                actual_player == board[new_x as usize][new_y as usize],
                                get_other_edge!(tuple_focused, way),
                                actual_take,
                                opp_align,
                                edge_opp,
                            );
                        }
                    }
                    if check == 1 {
                        ret.push((x, y, score));
                    }
                }
            }
        }
    }
    ret.sort_by(|(_, _, score1), (_, _, score2)| score2.cmp(score1));
    ret
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

fn board_state_win(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    actual_take: &mut isize,
    opp_take: &mut isize,
) -> bool {
    if *actual_take >= 5 || *opp_take >= 5 {
        return true;
    }
    let score_board = heuristic::evaluate_board(board);
    for x in 0..SIZE_BOARD {
        for y in 0..SIZE_BOARD {
            let mut can_take = false;
            let mut winner_align = false;
            for dir in 0..4 {
                let focused_tuple = score_board[x][y][dir];
                if winner_align || focused_tuple.0 >= 5 {
                    winner_align = true;
                }
                if can_take
                    || (focused_tuple.0 == 2
                        && ((focused_tuple.1 == Some(false) && focused_tuple.2 == Some(true))
                            || (focused_tuple.1 == Some(true) && focused_tuple.2 == Some(false))))
                {
                    can_take = true;
                }
            }
            if winner_align && !can_take {
                return true;
            }
        }
    }
    false
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
    color: &mut i8,
) -> (i64, Option<(usize, usize)>) {
    // println!("entry: {}", current_depth);
    if *current_depth == DEPTH_MAX || board_state_win(board, actual_catch, opp_catch)
    //    if *current_depth == DEPTH_MAX
    //        || winner_move!(board, last_move)
    //        || *actual_catch >= 5
    //        || *opp_catch >= 5
    {
        let weight = heuristic::first_heuristic_hint(
            board,
            actual,
            actual_catch,
            opp_catch,
            &mut (DEPTH_MAX - *current_depth),
        );
        // let lol = 10;
        //        println!(
        //            "evaluation - first print | catch:{} | depth: {}| heur: {}",
        //            actual_catch, current_depth, (lol * (*color as i64))
        //        );
        //        for i in 0..19 {
        //            for j in 0..19 {
        //                match board[j][i] {
        //                    Some(true) => print!("⊖"),
        //                    Some(false) => print!("⊕"),
        //                    None => print!("_"),
        //                }
        //            }
        //            //            println!();
        //        }
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
        return (weight, None);
        // return (10, None);
    }

    // Otherwise bubble up values from below
    let mut best_move: Option<(usize, usize)> = None;
    let mut best_score = MIN_INFINITY;

    // Collect moves
    let available_positions = get_space(board, actual, *actual_catch);
    //    if available_positions.len() > 0 {
    //        let (x, y, _) = available_positions[0];
    //        return (0, Some((x, y)));
    //    }
    println!("--------------");
    println!("Board state");
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
    available_positions
        .iter()
        .for_each(|&(x, y, score)| print!("({},{},{})//", x, y, score));
    println!();
    // let available_positions2 = get_space!(board, actual);

    // Go through each move
    for (line, col, _) in available_positions {
        // // debug
        // if board[line][col] != None {
        //     unreachable!();
        // }
        //println!("--------------------------");
        //        println!(
        //            "board - first print | catch:{} | depth: {}",
        //            actual_catch, current_depth
        //        );
        //        for i in 0..19 {
        //            for j in 0..19 {
        //                match board[j][i] {
        //                    Some(true) => print!("⊖"),
        //                    Some(false) => print!("⊕"),
        //                    None => print!("_"),
        //                }
        //            }
        //            println!();
        //        }
        //
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

        //        println!(
        //            "board - after change | catch:{} | depth: {}",
        //            *actual_catch, current_depth
        //        );
        //        for i in 0..19 {
        //            for j in 0..19 {
        //                match board[j][i] {
        //                    Some(true) => print!("⊖"),
        //                    Some(false) => print!("⊕"),
        //                    None => print!("_"),
        //                }
        //            }
        //            println!();
        //        }

        // Recurse
        let (recursed_score, _) = ab_negamax(
            board,
            table,
            zhash,
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

        //        println!(
        //            "board - after repair | catch:{} | depth: {} | current_score: {}",
        //            *actual_catch,
        //            current_depth,
        //            recursed_score * test.pow((*current_depth) as u32)
        //        );
        //        for i in 0..19 {
        //            for j in 0..19 {
        //                match board[j][i] {
        //                    Some(true) => print!("⊖"),
        //                    Some(false) => print!("⊕"),
        //                    None => print!("_"),
        //                }
        //            }
        //            //            println!();
        //        }

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
    //    println!("ENTRY RECURS");
    let (_, r#move): (i64, Option<(usize, usize)>) = ab_negamax(
        board,
        table,
        zhash,
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
        _ => unreachable!(),
    }
}

fn ia(
    game: &mut game::Game,
    (table, mut hash): ([[[u64; 2]; SIZE_BOARD]; SIZE_BOARD], u64),
) -> (usize, usize) {
    //    let player = game.get_actual_player();
    let mut player_catch = game.get_actual_player().nb_of_catch;
    let mut opponent_catch = game.get_opponent().nb_of_catch;
    let mut board = game.board;
    let pawn = game.player_to_pawn();
    match pawn {
        Some(true) => println!("New call : Blanc"),
        Some(false) => println!("New call : Noir"),
        _ => unreachable!(),
    }

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
            //            println!("move found");
            //            println!("board without move :");
            //            for i in 0..19 {
            //                for j in 0..19 {
            //                    match game.board[j][i] {
            //                        Some(true) => print!("⊖"),
            //                        Some(false) => print!("⊕"),
            //                        None => print!("_"),
            //                    }
            //                }
            //                //                println!();
            //            }
            ret
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn test_win(
        white_pos: Vec<(usize, usize)>,
        black_pos: Vec<(usize, usize)>,
        actual_take: &mut isize,
        opp_take: &mut isize,
    ) -> bool {
        let mut test_board = [[None; SIZE_BOARD]; SIZE_BOARD];
        white_pos
            .iter()
            .for_each(|&(x, y)| test_board[x][y] = Some(true));
        black_pos
            .iter()
            .for_each(|&(x, y)| test_board[x][y] = Some(false));
        for i in 0..19 {
            for j in 0..19 {
                match test_board[j][i] {
                    Some(true) => print!("⊖"),
                    Some(false) => print!("⊕"),
                    None => print!("_"),
                }
            }
            println!();
        }
        board_state_win(&mut test_board, actual_take, opp_take)
    }

    #[test]
    fn win_take0() {
        let black_pos = vec![];
        let white_pos = vec![];
        let mut white_take = 0isize;
        let mut black_take = 5isize;
        assert!(test_win(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take
        ))
    }
    #[test]
    fn win_take1() {
        let black_pos = vec![];
        let white_pos = vec![];
        let mut white_take = 5isize;
        let mut black_take = 0isize;
        assert!(test_win(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take
        ))
    }
}
