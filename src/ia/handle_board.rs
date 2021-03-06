use super::super::checks::after_turn_check::{check_explore, DIRECTIONS};
use super::super::checks::capture::get_capture_vec_hint;
use super::super::checks::capture::DIRS;
use super::super::checks::double_three::check_double_three_hint;
use super::super::model::board::Board;
use super::super::model::bool_option::get_opp;
use super::super::model::params::ParamsIA;
use super::super::model::score_board::ScoreBoard;
use super::super::render::board::SIZE_BOARD;
use super::get_ia;
use super::heuristic;
use super::threat_space::threat_search_space;
use super::threat_space::TypeOfThreat;
use super::zobrist;

const SCORE_ALIGN: i64 = 100;
const SCORE_TAKE: i64 = 100;
const MAX_INFINITY: i64 = i64::max_value();

// const MIN_INFINITY: i64 = i64::min_value() + 1;
// const MAX_INFINITY: i64 = i64::max_value();

macro_rules! valid_coord {
    ($x: expr, $y: expr) => {
        $x >= 0 && $x < SIZE_BOARD as isize && $y >= 0 && $y < SIZE_BOARD as isize
    };
}

// pub fn print_board(board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD]) {
//     for i in 0..19 {
//         print!("// ");
//         for j in 0..19 {
//             match board[j][i] {
//                 Some(true) => print!("⊖"),
//                 Some(false) => print!("⊕"),
//                 None => print!("_"),
//             }
//         }
//         println!();
//     }
// }
// fn print_board_new_add(board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD], x: usize, y: usize) {
//     for i in 0..19 {
//         print!("// ");
//         for j in 0..19 {
//             if i == y && j == x {
//                 print!("⊛");
//                 continue;
//             }
//             match board[j][i] {
//                 Some(true) => print!("⊖"),
//                 Some(false) => print!("⊕"),
//                 None => print!("_"),
//             }
//         }
//         println!();
//     }
// }

// Modify
fn modify_align(
    score_board: &mut ScoreBoard,
    (x, y): (usize, usize),
    (dx, dy): (&isize, &isize),
    dir: usize,
    len_change: u8,
    len_align: u8,
    left_edge: Option<bool>,
    right_edge: Option<bool>,
) {
    let mut new_x = x as isize;
    let mut new_y = y as isize;
    for _ in 0..len_change {
        new_x += dx;
        new_y += dy;
        if !valid_coord!(new_x, new_y) {
            score_board.print();
            println!(
                "start : ({},{})\ndirection : ({},{})\nlen_to_change : {}\nlen_to_put : {}",
                x, y, dx, dy, len_change, len_align
            );
        }
        score_board.set(
            new_x as usize,
            new_y as usize,
            dir,
            (len_align, left_edge, right_edge),
        );
    }
}

fn decrease_align(
    board: &mut Board,
    score_board: &mut ScoreBoard,
    pawn: Option<bool>,
    (x, y): (isize, isize),
    (dx, dy): (&isize, &isize),
    dir: usize,
    left_edge: Option<bool>,
    right_edge: Option<bool>,
) {
    let mut new_x = x;
    let mut new_y = y;
    let mut to_change = Vec::with_capacity(10);
    let mut nbr_align = 0u8;
    loop {
        new_x += dx;
        new_y += dy;
        match board.get(new_x as usize, new_y as usize) {
            Some(a) if a == pawn => {
                to_change.push((new_x, new_y));
                nbr_align += 1;
            }
            _ => {
                for &(old_x, old_y) in to_change.iter() {
                    //                println!("change here");
                    score_board.set(
                        old_x as usize,
                        old_y as usize,
                        dir,
                        (nbr_align, left_edge, right_edge),
                    )
                }
                break;
            }
        }
    }
}

// Change score board after the removal of a pawn
fn change_score_board_remove(board: &mut Board, score_board: &mut ScoreBoard, x: isize, y: isize) {
    let pawn = board.get_pawn(x as usize, y as usize);
    board.set(x as usize, y as usize, None);

    for (i, (dx, dy)) in DIRECTIONS.iter().enumerate() {
        score_board.reset(x as usize, y as usize, i);
        for way in [-1, 1].iter() {
            let new_x = x + (way * dx);
            let new_y = y + (way * dy);
            match board.get(new_x as usize, new_y as usize) {
                Some(None) => (),
                Some(a) if a == pawn => {
                    let (_, mut left_edge, mut right_edge) =
                        score_board.get(new_x as usize, new_y as usize, i);
                    if *way == -1 {
                        right_edge = Some(false);
                    } else {
                        left_edge = Some(false);
                    }
                    decrease_align(
                        board,
                        score_board,
                        pawn,
                        (x, y),
                        (&(way * dx), &(way * dy)),
                        i,
                        left_edge,
                        right_edge,
                    );
                }
                Some(a) if a != pawn => {
                    let (align, mut left_edge, mut right_edge) =
                        score_board.get(new_x as usize, new_y as usize, i);
                    if *way == -1 {
                        right_edge = Some(false);
                    } else {
                        left_edge = Some(false);
                    }
                    if align >= 10 {
                        println!("wtf2");
                    }
                    modify_align(
                        score_board,
                        (x as usize, y as usize),
                        (&(way * dx), &(way * dy)),
                        i,
                        align,
                        align,
                        left_edge,
                        right_edge,
                    );
                }
                Some(_) => unreachable!(),
                None => (),
            }
        }
    }
}

// Change score_board when add a pawn
pub fn change_score_board_add(
    board: &mut Board,
    score_board: &mut ScoreBoard,
    x: usize,
    y: usize,
    pawn: Option<bool>,
) {
    board.set(x, y, pawn);
    if pawn == None {
        println!("Pas normal");
        unreachable!();
    }
    let get_edge_case = |actual: &Option<bool>, to_cmp: Option<bool>| match to_cmp {
        None => return 1,
        a if a == *actual => return 2,
        a if a != *actual => return 3,
        _ => unreachable!(),
    };
    //iter through eveery direction to change the alignement value
    for (i, (dx, dy)) in DIRECTIONS.iter().enumerate() {
        let mut tot_align = 1;
        let x_left = x as isize - dx;
        let y_left = y as isize - dy;

        // Get case on the left hand side of the align
        // (board edge, empty slot, ally pawnm ennemy pawn)
        let edge_case_left = match board.get(x_left as usize, y_left as usize) {
            None => 0,
            Some(a) => get_edge_case(&pawn, a),
        };
        let (mut align_left, mut left_edge_left): (u8, Option<bool>) = (0, None);
        match edge_case_left {
            // edge
            0 => (),
            // empty slot
            1 => left_edge_left = Some(false),
            // align ally
            2 => {
                let focused_tuple = score_board.get(x_left as usize, y_left as usize, i);
                align_left = focused_tuple.0;
                left_edge_left = focused_tuple.1;
                tot_align += align_left;
            }
            // ennemy align
            3 => {
                align_left = score_board.get(x_left as usize, y_left as usize, i).0;
                left_edge_left = Some(true);
            }
            _ => unreachable!(),
        };

        let x_right = x as isize + dx;
        let y_right = y as isize + dy;
        // Get case on the right hand side of the align
        // (board edge, empty slot, ally pawnm ennemy pawn)
        let edge_case_right = match board.get(x_right as usize, y_right as usize) {
            None => 0,
            Some(a) => get_edge_case(&pawn, a),
        };
        let (mut align_right, mut right_edge_right): (u8, Option<bool>) = (0, None);
        match edge_case_right {
            // edge
            0 => (),
            //empty slot
            1 => {
                right_edge_right = Some(false);
            }
            // align ally
            2 => {
                let focused_tuple = score_board.get(x_right as usize, y_right as usize, i);
                align_right = focused_tuple.0;
                right_edge_right = focused_tuple.2;
                tot_align += align_right;
            }
            // ennemy align
            3 => {
                align_right = score_board.get(x_right as usize, y_right as usize, i).0;
                right_edge_right = Some(true);
            }
            _ => unreachable!(),
        };

        score_board.set(
            x as usize,
            y as usize,
            i,
            (tot_align, left_edge_left, right_edge_right),
        );

        match edge_case_left {
            // Modify the left hand side ally align after placing a new pawn
            2 => modify_align(
                score_board,
                (x, y),
                (&-dx, &-dy),
                i,
                align_left,
                tot_align,
                left_edge_left,
                right_edge_right,
            ),
            // Modify the left hand side ennemy align edge after placing a new pawn
            3 => modify_align(
                score_board,
                (x, y),
                (&-dx, &-dy),
                i,
                align_left,
                align_left,
                score_board.get(x_left as usize, y_left as usize, i).1,
                Some(true),
            ),
            0..=1 => (),
            _ => unreachable!(),
        }

        match edge_case_right {
            // Modify the left hand side ally align after placing a new pawn
            2 => modify_align(
                score_board,
                (x, y),
                (dx, dy),
                i,
                align_right,
                tot_align,
                left_edge_left,
                right_edge_right,
            ),
            // Modify the left hand side ennemy align edge after placing a new pawn
            3 => modify_align(
                score_board,
                (x, y),
                (dx, dy),
                i,
                align_right,
                align_right,
                Some(true),
                score_board.get(x_right as usize, y_right as usize, i).2,
            ),
            0..=1 => (),
            _ => unreachable!(),
        }
    }
}

fn get_score_board(
    align: u8,
    is_same_pawn: bool,
    edge: Option<bool>,
    nb_take: isize,
    align_opp: u8,
    edge_opp: Option<bool>,
) -> i64 {
    match is_same_pawn {
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
            if tot_align == 2 && edged == 1 {
                return -SCORE_ALIGN.pow(3);
            }
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
                //both side close with boarda
                6 => SCORE_ALIGN.pow(tot_align as u32) / 16,
                //close *2
                _ => 0,
            }
        }
    }
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

fn best_of_board(
    params: &mut ParamsIA,
    lst_moove: Vec<((usize, usize), TypeOfThreat, std::vec::Vec<(usize, usize)>)>,
) -> Option<(usize, usize)> {
    let mut best_move = (MAX_INFINITY, TypeOfThreat::ThreeOF, (0, 0));

    lst_moove.iter().for_each(|&((line, col), threat, _)| {
        if threat >= best_move.1 {
            let removed = change_board(
                &mut params.board,
                &mut params.score_board,
                line,
                col,
                params.actual,
                &mut params.zhash,
            );
            params.actual_catch += removed.len() as isize;

            let tmp_move = get_ia::iterative_deepening_mtdf(params, false);

            best_move = match (best_move, tmp_move) {
                (x, y) if x.0 <= y.0 => x,
                (x, y) if x.0 > y.0 => (y.0, threat, (line, col)),
                (_, _) => unreachable!(),
            };

            params.actual_catch -= removed.len() as isize;
            remove_last_pawn(
                &mut params.board,
                &mut params.score_board,
                line,
                col,
                params.actual,
                removed,
                &mut params.zhash,
            );
        }
    });

    Some(best_move.2)
}

pub fn get_space(
    board: &mut Board,
    score_board: &mut ScoreBoard,
    actual_player: Option<bool>,
    actual_take: isize,
) -> Vec<(usize, usize, i64)> {
    let mut ret = Vec::with_capacity(200);
    for x in 0..SIZE_BOARD {
        for y in 0..SIZE_BOARD {
            let value = board.get_pawn(x, y);
            if value == None {
                let mut check = 0;
                let mut to_add = false;
                let mut score = 0i64;

                for &(dx, dy) in DIRS.iter() {
                    let new_x = x as isize + dx;
                    let new_y = y as isize + dy;
                    match board.get(new_x as usize, new_y as usize) {
                        None => (),
                        Some(None) => (),
                        Some(Some(_)) => {
                            if check != 0
                                || !check_double_three_hint(
                                    board,
                                    actual_player,
                                    x as isize,
                                    y as isize,
                                )
                            {
                                let mut edge_opp = None;
                                let mut opp_align = 0;
                                let opp_x = x as isize - dx;
                                let opp_y = y as isize - dx;
                                let (dir, way) = get_dir!((dx, dy));
                                match board.get(opp_x as usize, opp_y as usize) {
                                    Some(a) if a == value => {
                                        let opp_tuple =
                                            score_board.get(opp_x as usize, opp_y as usize, dir);
                                        opp_align = opp_tuple.0;
                                        edge_opp = get_other_edge!(opp_tuple, (way + 1) % 2);
                                        to_add = true;
                                    }
                                    Some(None) => {
                                        edge_opp = Some(false);
                                    }
                                    Some(a) if a != value => {
                                        edge_opp = Some(true);
                                        to_add = true;
                                    }
                                    Some(_) => unreachable!(),
                                    None => (),
                                }
                                check = 1;
                                let tuple_focused =
                                    score_board.get(new_x as usize, new_y as usize, dir);
                                score += get_score_board(
                                    tuple_focused.0,
                                    actual_player == board.get_pawn(new_x as usize, new_y as usize),
                                    get_other_edge!(tuple_focused, way),
                                    actual_take,
                                    opp_align,
                                    edge_opp,
                                );
                            }
                        }
                    }
                }

                if to_add {
                    ret.push((x, y, score));
                }
            }
        }
    }
    ret.sort_by(|(_, _, score1), (_, _, score2)| score2.cmp(score1));
    ret
}

pub fn null_move_heuristic(params: &mut ParamsIA) -> Option<(usize, usize)> {
    let mut actual_threat = threat_search_space(
        &mut params.board,
        &mut params.score_board,
        params.actual,
        &mut params.actual_catch,
    );
    let mut opp_threat = threat_search_space(
        &mut params.board,
        &mut params.score_board,
        get_opp(params.actual),
        &mut params.opp_catch,
    );
    opp_threat = opp_threat
        .into_iter()
        .filter(|((x, y), _, _)| {
            !check_double_three_hint(&mut params.board, params.actual, *x as isize, *y as isize)
        })
        .collect::<Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)>>();
    actual_threat = actual_threat
        .into_iter()
        .filter(|((x, y), _, _)| {
            !check_double_three_hint(&mut params.board, params.actual, *x as isize, *y as isize)
        })
        .collect::<Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)>>();
    if actual_threat.len() == 0 {
        return None;
    }
    if opp_threat.len() == 0 || opp_threat[0].1 < TypeOfThreat::FourSOF {
        return None;
    } else if opp_threat[0].1 == TypeOfThreat::AlreadyWon {
        for line in 0..19 {
            for col in 0..19 {
                for dir in 0..4 {
                    if params.score_board.get(line, col, dir).0 >= 5 {
                        let (dx, dy) = DIRECTIONS[dir];
                        if let Some(to_take) = check_explore(
                            &mut params.board.into(),
                            (dx, dy),
                            &(line as isize, col as isize),
                        ) {
                            let captures = get_capture_vec_hint(
                                &mut params.board,
                                &mut params.score_board,
                                to_take,
                            );
                            if captures.len() == 0 {
                                return None;
                            } else {
                                return Some(captures[0]);
                            }
                        } else {
                            return None;
                        }
                    }
                }
            }
        }
        return None;
    } else if actual_threat.len() == 0 {
        return best_of_board(params, opp_threat);
    } else {
        if actual_threat[0].1 >= opp_threat[0].1 {
            return None;
        } else if opp_threat[0].1 >= TypeOfThreat::FourSOF {
            return best_of_board(params, opp_threat);
        } else {
            return None;
        }
    }
}
pub fn find_continuous_threats(
    params: &mut ParamsIA,
    depth: &mut i8,
    depth_win: &mut i8,
    true_actual_player: bool,
    mainloop: bool,
) -> Option<(usize, usize)> {
    if *depth < *depth_win || params.check_timeout(mainloop) {
        return None;
    }
    let threats: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = threat_search_space(
        &mut params.board,
        &mut params.score_board,
        params.actual,
        &mut params.actual_catch,
    );

    if threats.len() == 0 {
        return None;
    }

    if threats.len() == 1 && threats[0].1 >= TypeOfThreat::WIN {
        *depth_win = *depth;
        if true_actual_player {
            return Some(threats[0].0);
        } else {
            return None;
        }
    }

    for (threat, typeofthreat, counters) in threats.iter() {
        if *typeofthreat != threats[0].1 {
            break;
        }
        let (x, y) = threat;

        let removed = change_board_hint(
            &mut params.board,
            &mut params.score_board,
            *x,
            *y,
            params.actual,
        );
        params.actual_catch += removed.len() as isize;

        let mut counters_valid: Vec<(usize, usize)> = vec![];
        for (opp_x, opp_y) in counters.iter() {
            if !check_double_three_hint(
                &mut params.board,
                get_opp(params.actual),
                *opp_x as isize,
                *opp_y as isize,
            ) {
                counters_valid.push((*opp_x, *opp_y));
            }
        }

        if counters_valid.len() == 0 {
            *depth_win = *depth;
            remove_last_pawn_hint(
                &mut params.board,
                &mut params.score_board,
                *x,
                *y,
                params.actual,
                removed,
            );
            return Some((*x, *y));
        }

        let mut win: bool = true;

        for (counter_x, counter_y) in counters_valid.iter() {
            if params.board.get_pawn(*counter_x, *counter_y) != None {
                unreachable!();
            }
            let removed_counter = change_board_hint(
                &mut params.board,
                &mut params.score_board,
                *counter_x,
                *counter_y,
                get_opp(params.actual),
            );

            params.opp_catch += removed_counter.len() as isize;
            let res = find_continuous_threats(
                params,
                &mut (*depth - 2),
                depth_win,
                true_actual_player,
                mainloop,
            );
            params.opp_catch -= removed_counter.len() as isize;
            remove_last_pawn_hint(
                &mut params.board,
                &mut params.score_board,
                *counter_x,
                *counter_y,
                get_opp(params.actual),
                removed_counter,
            );

            if res == None {
                win = false;
                break;
            }
        }
        params.actual_catch -= removed.len() as isize;
        remove_last_pawn_hint(
            &mut params.board,
            &mut params.score_board,
            *x,
            *y,
            params.actual,
            removed,
        );

        if win {
            *depth_win = *depth;
            return Some((*x, *y));
        }
    }
    None
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

macro_rules! add_zhash {
    ($zhash:expr, $x:expr, $y:expr, $piece:expr) => {
        unsafe {
            *$zhash ^= zobrist::ZTABLE[$x as usize][$y as usize][zobrist::ZPIECES[$piece]];
        }
    };
}

pub fn remove_last_pawn_hint(
    board: &mut Board,
    score_board: &mut ScoreBoard,
    x: usize,
    y: usize,
    pawn: Option<bool>,
    removed: Vec<((isize, isize), (isize, isize))>,
) {
    let old = get_opp(pawn);
    change_score_board_remove(board, score_board, x as isize, y as isize);
    removed.iter().for_each(|&((x1, y1), (x2, y2))| {
        change_score_board_add(board, score_board, x1 as usize, y1 as usize, old);
        change_score_board_add(board, score_board, x2 as usize, y2 as usize, old);
    })
}

pub fn change_board_hint(
    board: &mut Board,
    score_board: &mut ScoreBoard,
    x: usize,
    y: usize,
    pawn: Option<bool>,
) -> Vec<((isize, isize), (isize, isize))> {
    let mut removed = Vec::with_capacity(16);
    change_score_board_add(board, score_board, x, y, pawn);
    let opp = get_opp(pawn);
    for &(dx, dy) in DIRS.iter() {
        let mut count = 0;
        let mut new_x = x as isize;
        let mut new_y = y as isize;
        for _ in 0..3 {
            new_x += dx;
            new_y += dy;
            match board.get(new_x as usize, new_y as usize) {
                Some(a) if a == opp => count += 1,
                Some(a) if a != opp => break,
                Some(_) => unreachable!(),

                None => {
                    count = 0;
                    break;
                }
            }
        }
        if count == 2 && board.get_pawn(new_x as usize, new_y as usize) == pawn {
            let (x1, y1) = (new_x - dx, new_y - dy);
            let (x2, y2) = (x1 - dx, y1 - dy);
            change_score_board_remove(board, score_board, x1 as isize, y1 as isize);
            change_score_board_remove(board, score_board, x2 as isize, y2 as isize);
            removed.push(((x1, y1), (x2, y2)));
        }
    }

    removed
}

pub fn remove_last_pawn(
    board: &mut Board,
    score_board: &mut ScoreBoard,
    x: usize,
    y: usize,
    pawn: Option<bool>,
    removed: Vec<((isize, isize), (isize, isize))>,
    zhash: &mut u64,
) {
    let old = get_opp(pawn);
    change_score_board_remove(board, score_board, x as isize, y as isize);
    add_zhash!(zhash, x, y, get_zindex_from_pawn!(pawn));
    removed.iter().for_each(|&((x1, y1), (x2, y2))| {
        add_zhash!(zhash, x1, y1, get_zindex_from_pawn!(old));
        change_score_board_add(board, score_board, x1 as usize, y1 as usize, old);
        add_zhash!(zhash, x2, y2, get_zindex_from_pawn!(old));
        change_score_board_add(board, score_board, x2 as usize, y2 as usize, old);
    });
}

pub fn change_board(
    board: &mut Board,
    score_board: &mut ScoreBoard,
    x: usize,
    y: usize,
    pawn: Option<bool>,
    zhash: &mut u64,
) -> Vec<((isize, isize), (isize, isize))> {
    let mut removed = Vec::with_capacity(16);
    change_score_board_add(board, score_board, x, y, pawn);
    add_zhash!(zhash, x, y, get_zindex_from_pawn!(pawn));
    let opp = get_opp(pawn);
    for &(dx, dy) in DIRS.iter() {
        let mut count = 0;
        let mut new_x = x as isize;
        let mut new_y = y as isize;
        for _ in 0..3 {
            new_x += dx;
            new_y += dy;
            match board.get(new_x as usize, new_y as usize) {
                Some(a) if a == opp => count += 1,
                Some(a) if a != opp => break,
                Some(_) => unreachable!(),

                None => {
                    count = 0;
                    break;
                }
            }
        }
        if count == 2 && board.get_pawn(new_x as usize, new_y as usize) == pawn {
            let (x1, y1) = (new_x - dx, new_y - dy);
            let (x2, y2) = (x1 - dx, y1 - dy);
            change_score_board_remove(board, score_board, x1, y1);
            add_zhash!(zhash, x1, y1, get_zindex_from_pawn!(opp));
            change_score_board_remove(board, score_board, x2, y2);
            add_zhash!(zhash, x2, y2, get_zindex_from_pawn!(opp));
            removed.push(((x1, y1), (x2, y2)));
        }
    }

    removed
}
#[cfg(test)]
mod tests {
    use super::*;

    pub fn board_state_win(
        score_board: &mut ScoreBoard,
        actual_take: &mut isize,
        opp_take: &mut isize,
    ) -> bool {
        if *actual_take >= 5 || *opp_take >= 5 {
            return true;
        }
        for x in 0..SIZE_BOARD {
            for y in 0..SIZE_BOARD {
                let mut can_take = false;
                let mut winner_align = false;
                let mut len_align = 0;
                let mut dir_align = 0;
                for dir in 0..4 {
                    let focused_tuple = score_board.get(x, y, dir);
                    if winner_align || focused_tuple.0 >= 5 {
                        len_align = focused_tuple.0;
                        dir_align = dir;
                        winner_align = true;
                    }
                    if can_take
                        || (focused_tuple.0 == 2
                            && ((focused_tuple.1 == Some(false) && focused_tuple.2 == Some(true))
                                || (focused_tuple.1 == Some(true)
                                    && focused_tuple.2 == Some(false))))
                    {
                        can_take = true;
                    }
                }
                if winner_align && !can_take {
                    let mut win = false;
                    let mut new_x = x as isize;
                    let mut new_y = y as isize;
                    let direction = DIRS[dir_align];
                    for way in [-1, 1].iter() {
                        loop {
                            new_x += way * direction.0;
                            new_y += way * direction.1;
                            match score_board.get_check(new_x as usize, new_y as usize, dir_align) {
                                Some(a) if a.0 == len_align => {
                                    for dir in 0..4 {
                                        if dir == dir_align {
                                            continue;
                                        }
                                        let focused_tuple =
                                            score_board.get(new_x as usize, new_y as usize, dir);
                                        if focused_tuple.0 == 2
                                            && ((focused_tuple.1 == Some(false)
                                                && focused_tuple.2 == Some(true))
                                                || (focused_tuple.1 == Some(true)
                                                    && focused_tuple.2 == Some(false)))
                                        {
                                            win = true;
                                            break;
                                        }
                                    }
                                }
                                _ => break,
                            }
                        }
                    }
                    if !win {
                        return win;
                    }
                }
            }
        }
        false
    }

    // pub fn find_available_pos(board: &mut Board, player_actual: Option<bool>) -> Vec<(usize, usize)> {
    //     let mut ret: Vec<(usize, usize)> = vec![];
    //     for x in 0..SIZE_BOARD {
    //         for y in 0..SIZE_BOARD {
    //             if board.get_pawn(x, y) == None {
    //                 for &(dx, dy) in DIRS.iter() {
    //                     let new_x = x as isize + dx;
    //                     let new_y = y as isize + dy;
    //                     if board.get_pawn(new_x as usize, new_y as usize) != None {
    //                         if !check_double_three_hint(board, player_actual, x as isize, y as isize) {
    //                             ret.push((x as usize, y as usize));
    //                         }
    //                         break;
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     ret
    // }

    // pub fn find_continuous_threats_print(
    //     board: &mut Board,
    //     score_board: &mut ScoreBoard,
    //     player_actual: Option<bool>,
    //     player_actual_catch: &mut isize,
    //     player_opposite_catch: &mut isize,
    //     depth: &mut i8,
    //     depth_win: &mut i8,
    // ) -> Option<(usize, usize)> {
    //     if *depth < *depth_win {
    //         return None;
    //     }
    //     let threats: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
    //         threat_search_space(board, score_board, player_actual, player_actual_catch);

    //     println!("hello : {} threat", threats.len());
    //     if threats.len() == 0 {
    //         return None;
    //     }

    //     if threats.len() == 1 && threats[0].1 == TypeOfThreat::WIN {
    //         *depth_win = *depth;
    //         let (x, y) = threats[0].0;
    //         println!("Winner threat in ({},{})", x, y);
    //         for i in 0..19 {
    //             for j in 0..19 {
    //                 if i == y && j == x {
    //                     print!("⊛")
    //                 } else {
    //                     match board.get_pawn(j, i) {
    //                         Some(true) => print!("⊖"),
    //                         Some(false) => print!("⊕"),
    //                         None => print!("_"),
    //                     }
    //                 }
    //             }
    //             println!();
    //         }
    //         return Some(threats[0].0);
    //     }

    //     for (threat, typeofthreat, counters) in threats.iter() {
    //         let (x, y) = threat;
    //         let removed = change_board_hint(board, score_board, *x, *y, player_actual);
    //         *player_actual_catch += removed.len() as isize;

    //         println!(
    //             "Config depth {}, black pawn in ({},{}), threat : {} ",
    //             depth,
    //             x,
    //             y,
    //             match typeofthreat {
    //                 TypeOfThreat::FiveTake => "FiveTake",
    //                 TypeOfThreat::FourTake => "FourTake",
    //                 TypeOfThreat::ThreeTake => "ThreeTake",
    //                 TypeOfThreat::TwoTake => "TwoTake",
    //                 TypeOfThreat::OneTake => "OneTake",
    //                 TypeOfThreat::FourOC => "FourOC",
    //                 TypeOfThreat::FourSOC => "FourSOC",
    //                 TypeOfThreat::FourSOF => "FourSOF",
    //                 TypeOfThreat::ThreeOC => "ThreeOC",
    //                 TypeOfThreat::ThreeOF => "ThreeOF",
    //                 TypeOfThreat::WIN => "WIN",
    //                 TypeOfThreat::AlreadyWon => "WON",
    //                 TypeOfThreat::EMPTY => "EMPTY",
    //             }
    //         );
    //         for i in 0..19 {
    //             for j in 0..19 {
    //                 if i == *y && j == *x {
    //                     print!("⊛")
    //                 } else {
    //                     match board.get_pawn(j, i) {
    //                         Some(true) => print!("⊖"),
    //                         Some(false) => print!("⊕"),
    //                         None => print!("_"),
    //                     }
    //                 }
    //             }
    //             println!();
    //         }
    //         let mut counters_valid: Vec<(usize, usize)> = vec![];
    //         for (opp_x, opp_y) in counters.iter() {
    //             if !check_double_three_hint(
    //                 board,
    //                 get_opp(player_actual),
    //                 *opp_x as isize,
    //                 *opp_y as isize,
    //             ) {
    //                 counters_valid.push((*opp_x, *opp_y));
    //             }
    //         }
    //         print!("Counters : ");
    //         counters_valid
    //             .iter()
    //             .for_each(|(c_x, c_y)| print!("({},{}); ", c_x, c_y));
    //         println!();

    //         if counters_valid.len() == 0 && *typeofthreat < TypeOfThreat::FiveTake {
    //             for (x, y) in find_available_pos(board, get_opp(player_actual)) {
    //                 if !counters_valid
    //                     .iter()
    //                     .any(|&(cmp_x, cmp_y)| cmp_x == x && cmp_y == y)
    //                 {
    //                     counters_valid.push((x, y));
    //                 }
    //             }
    //         }

    //         print!("Counters : ");
    //         counters_valid
    //             .iter()
    //             .for_each(|(c_x, c_y)| print!("({},{}); ", c_x, c_y));
    //         println!();

    //         let mut win: bool = true;

    //         for (counter_x, counter_y) in counters_valid.iter() {
    //             let removed_counter = change_board_hint(
    //                 board,
    //                 score_board,
    //                 *counter_x,
    //                 *counter_y,
    //                 get_opp(player_actual),
    //             );
    //             println!(
    //                 "Config depth {}, white pawn in ({},{}) ",
    //                 depth, counter_x, counter_y
    //             );
    //             for i in 0..19 {
    //                 for j in 0..19 {
    //                     if i == *counter_y && j == *counter_x {
    //                         print!("⊙")
    //                     } else {
    //                         match board.get_pawn(j, i) {
    //                             Some(true) => print!("⊖"),
    //                             Some(false) => print!("⊕"),
    //                             None => print!("_"),
    //                         }
    //                     }
    //                 }
    //                 println!();
    //             }

    //             *player_opposite_catch += removed_counter.len() as isize;
    //             let res = find_continuous_threats(
    //                 board,
    //                 score_board,
    //                 player_actual,
    //                 player_actual_catch,
    //                 player_opposite_catch,
    //                 &mut (*depth - 2),
    //                 depth_win,
    //                 true,
    //             );
    //             *player_opposite_catch -= removed_counter.len() as isize;
    //             remove_last_pawn_hint(
    //                 board,
    //                 score_board,
    //                 *counter_x,
    //                 *counter_y,
    //                 get_opp(player_actual),
    //                 removed_counter,
    //             );
    //             println!();
    //             println!();

    //             if res == None {
    //                 win = false;
    //             }
    //         }
    //         *player_actual_catch -= removed.len() as isize;
    //         remove_last_pawn_hint(board, score_board, *x, *y, player_actual, removed);

    //         if win {
    //             println!("Winner threat in ({},{})", *x, *y);
    //             for i in 0..19 {
    //                 for j in 0..19 {
    //                     if i == *y && j == *x {
    //                         print!("⊛")
    //                     } else {
    //                         match board.get_pawn(j, i) {
    //                             Some(true) => print!("⊖"),
    //                             Some(false) => print!("⊕"),
    //                             None => print!("_"),
    //                         }
    //                     }
    //                 }
    //                 println!();
    //             }
    //             *depth_win = *depth;
    //             return Some((*x, *y));
    //         }
    //     }
    //     None
    // }

    macro_rules! get_bool {
        ($e:expr) => {
            match $e {
                Some(true) => "T",
                Some(false) => "F",
                None => "N",
            }
        };
    }
    fn test_win(
        white_pos: Vec<(usize, usize)>,
        black_pos: Vec<(usize, usize)>,
        actual_take: &mut isize,
        opp_take: &mut isize,
    ) -> bool {
        let mut test_board: Board = [[None; SIZE_BOARD]; SIZE_BOARD].into();
        let mut score_tab: ScoreBoard =
            [[[(0, Some(false), Some(false)); 4]; SIZE_BOARD]; SIZE_BOARD].into();
        white_pos.iter().for_each(|&(x, y)| {
            test_board.set(x, y, Some(true));
            change_score_board_add(&mut test_board, &mut score_tab, x, y, Some(true));
        });
        black_pos.iter().for_each(|&(x, y)| {
            test_board.set(x, y, Some(false));
            change_score_board_add(&mut test_board, &mut score_tab, x, y, Some(false));
        });
        test_board.print();
        for i in 0..19 {
            for j in 0..19 {
                match test_board.get_pawn(j, i) {
                    Some(true) => print!("B"),
                    Some(false) => print!("N"),
                    None => print!("E"),
                }
                score_tab.get_arr(j, i).iter().for_each(|&(value, a, b)| {
                    print!("{:2}{}{}", value, get_bool!(a), get_bool!(b))
                });
                print!(" ");
            }
            println!();
        }
        board_state_win(&mut score_tab, actual_take, opp_take)
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
    #[test]
    fn win_align0() {
        let black_pos = vec![
            (6, 6),
            (4, 8),
            (4, 10),
            (7, 9),
            (8, 10),
            (8, 12),
            (9, 11),
            (9, 8),
            (10, 7),
            (10, 10),
            (11, 7),
            (12, 9),
            (14, 12),
        ];
        let white_pos = vec![
            (7, 5),
            (7, 7),
            (8, 8),
            (9, 9),
            (9, 7),
            (10, 8),
            (11, 9),
            (12, 10),
            (13, 11),
            (6, 8),
            (5, 9),
            (6, 8),
            (7, 9),
        ];
        let mut white_take = 0isize;
        let mut black_take = 0isize;
        assert!(!test_win(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take
        ))
    }

    #[test]
    fn test0() {
        let black_pos = vec![(8, 8)];
        let white_pos = vec![
            (9, 9),
            (9, 8),
            (9, 7),
            (8, 9),
            (8, 7),
            (7, 9),
            (7, 8),
            (7, 7),
        ];
        let mut white_take = 0isize;
        let mut black_take = 0isize;
        assert!(!test_win(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take
        ))
    }

    fn test_get_space(
        white_pos: Vec<(usize, usize)>,
        black_pos: Vec<(usize, usize)>,
        actual_take: &mut isize,
        actual_player: Option<bool>,
        expected_result: Vec<(usize, usize, i64)>,
    ) -> bool {
        let mut test_board_tmp = [[None; SIZE_BOARD]; SIZE_BOARD];
        let mut test_bboard = [[None; SIZE_BOARD]; SIZE_BOARD];
        white_pos
            .iter()
            .for_each(|&(x, y)| test_board_tmp[x][y] = Some(1));
        black_pos
            .iter()
            .for_each(|&(x, y)| test_board_tmp[x][y] = Some(0));
        white_pos
            .iter()
            .for_each(|&(x, y)| test_bboard[x][y] = Some(true));
        black_pos
            .iter()
            .for_each(|&(x, y)| test_bboard[x][y] = Some(false));

        let mut test_board: Board = test_bboard.into();
        // Print initial configuration
        println!("// Initial configuration:");
        test_board.print();
        let mut score_board = heuristic::evaluate_board(&mut test_board);

        let ret = get_space(
            &mut test_board,
            &mut score_board,
            actual_player,
            *actual_take,
        );

        ret.iter().for_each(|&(x, y, _)| {
            test_board_tmp[x][y] = Some(2);
        });

        println!("\n// Response:");
        for i in 0..19 {
            print!("// ");
            for j in 0..19 {
                match test_board_tmp[j][i] {
                    Some(2) => print!("⊙"),
                    Some(1) => print!("⊖"),
                    Some(0) => print!("⊕"),
                    None => print!("_"),
                    Some(_) => (),
                }
            }
            println!();
        }

        ret.iter().for_each(|(x, y, z)| {
            println!("output: ({},{},{})", x, y, z);
        });

        ret == expected_result
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊖_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Response:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ________⊙⊙⊙________
    // ________⊙⊕⊙________
    // ________⊙⊕⊙________
    // ________⊙⊕⊙________
    // ________⊙⊖⊙________
    // ________⊙⊙⊙________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn threat_get_space_1() {
        let black_pos = vec![(9, 8), (9, 7), (9, 9)];
        let white_pos = vec![(9, 10)];
        let mut white_take = 0_isize;
        let expected_result = vec![
            (9, 6, 500000),
            (8, 8, 300),
            (10, 8, 300),
            (8, 9, 250),
            (10, 9, 250),
            (8, 7, 200),
            (9, 11, 200),
            (10, 7, 200),
            (8, 10, 150),
            (10, 10, 150),
            (8, 6, 100),
            (10, 6, 100),
            (8, 11, 50),
            (10, 11, 50),
        ];

        assert!(test_get_space(
            white_pos,
            black_pos,
            &mut white_take,
            Some(false),
            expected_result
        ))
    }

    #[test]
    fn threat_get_space_2() {
        let black_pos = vec![(9, 8), (9, 7), (9, 9), (7, 7)];
        let white_pos = vec![(9, 10)];
        let mut white_take = 0_isize;
        let expected_result = vec![
            (9, 6, 500000),
            (10, 8, 300),
            (8, 7, 250),
            (8, 9, 250),
            (10, 9, 250),
            (8, 8, 200),
            (9, 11, 200),
            (10, 7, 200),
            (8, 6, 150),
            (8, 10, 150),
            (10, 10, 150),
            (6, 6, 100),
            (6, 7, 100),
            (6, 8, 100),
            (7, 6, 100),
            (7, 8, 100),
            (10, 6, 100),
            (8, 11, 50),
            (10, 11, 50),
        ];

        assert!(test_get_space(
            white_pos,
            black_pos,
            &mut white_take,
            Some(false),
            expected_result
        ))
    }

    use std::time::Instant;

    fn test_continuous_threats(
        white_pos: Vec<(usize, usize)>,
        black_pos: Vec<(usize, usize)>,
        actual_player: Option<bool>,
        actual_take: &mut isize,
        opp_take: &mut isize,
        depth: &mut i8,
        depth_win: &mut i8,
        intented: Option<(usize, usize)>,
    ) -> bool {
        let mut test_bboard = [[None; SIZE_BOARD]; SIZE_BOARD];
        white_pos
            .iter()
            .for_each(|&(x, y)| test_bboard[x][y] = Some(true));
        black_pos
            .iter()
            .for_each(|&(x, y)| test_bboard[x][y] = Some(false));
        let mut test_board: Board = test_bboard.into();
        let start_time = Instant::now();
        let mut params = ParamsIA {
            score_board: heuristic::evaluate_board(&mut test_board),
            board: test_board,
            zhash: zobrist::board_to_zhash(&test_bboard),
            current_depth: 0,
            actual: actual_player,
            actual_catch: *actual_take,
            opp_catch: *opp_take,
            alpha: i64::min_value() + 1,
            beta: i64::max_value(),
            color: 0,
            depth_max: 11,
            counter_tree: 0,
            start_time: start_time,
            f: 0,
            counter: 0,
        };

        // Print initial configuration
        println!("// Initial configuration:");
        test_board.print();

        let res = find_continuous_threats(&mut params, depth, depth_win, true, true);
        match res {
            None => println!("No threat found"),
            Some((x, y)) => println!("Threat found {}:{}", x, y),
        };

        res == intented
    }

    // fn test_continuous_threats_print(
    //     white_pos: Vec<(usize, usize)>,
    //     black_pos: Vec<(usize, usize)>,
    //     actual_player: Option<bool>,
    //     actual_take: &mut isize,
    //     opp_take: &mut isize,
    //     depth: &mut i8,
    //     depth_win: &mut i8,
    //     intented: Option<(usize, usize)>,
    // ) -> bool {
    //     let mut test_bboard = [[None; SIZE_BOARD]; SIZE_BOARD];
    //     white_pos
    //         .iter()
    //         .for_each(|&(x, y)| test_bboard[x][y] = Some(true));
    //     black_pos
    //         .iter()
    //         .for_each(|&(x, y)| test_bboard[x][y] = Some(false));
    //     let mut test_board: Board = test_bboard.into();
    //     let mut score_board = heuristic::evaluate_board(&mut test_board);
    //     // Print initial configuration
    //     println!("// Initial configuration:");
    //     test_board.print();
    //     let res = find_continuous_threats_print(
    //         &mut test_board,
    //         &mut score_board,
    //         actual_player,
    //         actual_take,
    //         opp_take,
    //         depth,
    //         depth_win,
    //     );
    //     match res {
    //         None => println!("No threat found"),
    //         Some((x, y)) => println!("Threat found {}:{}", x, y),
    //     };

    //     res == intented
    // }

    #[test]
    fn test_threat_simple_00() {
        let black_pos = vec![(9, 8), (9, 7), (9, 6)];
        let white_pos = vec![];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let mut depth = 2_i8;
        let mut depth_max = 0_i8;
        let expected_result = Some((9, 5));

        assert!(test_continuous_threats(
            white_pos,
            black_pos,
            Some(false),
            &mut black_take,
            &mut white_take,
            &mut depth,
            &mut depth_max,
            expected_result
        ))
    }

    #[test]
    fn test_threat_no_threat_00() {
        let black_pos = vec![(9, 8), (9, 7), (9, 6), (8, 4)];
        let white_pos = vec![(9, 9), (7, 3)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let mut depth = 12_i8;
        let mut depth_max = 0_i8;
        let expected_result = None;

        assert!(test_continuous_threats(
            white_pos,
            black_pos,
            Some(false),
            &mut black_take,
            &mut white_take,
            &mut depth,
            &mut depth_max,
            expected_result
        ))
    }

    #[test]
    fn test_threat_no_threat_01() {
        let black_pos = vec![(9, 8), (9, 7), (9, 6), (10, 6), (11, 7), (8, 5), (7, 5)];
        let white_pos = vec![(9, 9), (7, 3)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let mut depth = 12_i8;
        let mut depth_max = 0_i8;
        let expected_result = None;

        assert!(test_continuous_threats(
            white_pos,
            black_pos,
            Some(false),
            &mut black_take,
            &mut white_take,
            &mut depth,
            &mut depth_max,
            expected_result
        ))
    }

    #[test]
    fn test_threat_five_take_00() {
        let black_pos = vec![(9, 8), (9, 7), (9, 6), (9, 5), (8, 4)];
        let white_pos = vec![(9, 9), (7, 3)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let mut depth = 12_i8;
        let mut depth_max = 0_i8;
        let expected_result = Some((9, 4));

        assert!(test_continuous_threats(
            white_pos,
            black_pos,
            Some(false),
            &mut black_take,
            &mut white_take,
            &mut depth,
            &mut depth_max,
            expected_result
        ))
    }

    // fn print_board(board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD]) {
    //     for i in 0..19 {
    //         print!("// ");
    //         for j in 0..19 {
    //             match board[j][i] {
    //                 Some(true) => print!("⊖"),
    //                 Some(false) => print!("⊕"),
    //                 None => print!("_"),
    //             }
    //         }
    //         println!();
    //     }
    // }

    // fn print_score_board(
    //     score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    // ) {
    //     for i in 0..19 {
    //         print!("// ");
    //         for j in 0..19 {
    //             for dir in 0..4 {
    //                 match score_board[j][i][dir].1 {
    //                     Some(true) => print!("#"),
    //                     None => print!("|"),
    //                     Some(false) => print!("_"),
    //                 }
    //                 print!("{}", score_board[j][i][dir].0);
    //                 match score_board[j][i][dir].2 {
    //                     Some(true) => print!("#"),
    //                     None => print!("|"),
    //                     Some(false) => print!("_"),
    //                 }
    //             }
    //             print!(" ");
    //         }
    //         println!();
    //     }
    // }

    fn test_add_board(
        white_pos: Vec<(usize, usize)>,
        black_pos: Vec<(usize, usize)>,
        to_add: (usize, usize),
        actual_player: Option<bool>,
    ) -> bool {
        let mut bboard = [[None; SIZE_BOARD]; SIZE_BOARD];
        zobrist::init_zboard();
        let mut zhash = 0u64;
        white_pos
            .iter()
            .for_each(|&(x, y)| bboard[x][y] = Some(true));
        black_pos
            .iter()
            .for_each(|&(x, y)| bboard[x][y] = Some(false));
        let mut board: Board = bboard.into();
        let mut score_board = heuristic::evaluate_board(&mut board);
        println!("// Initial configuration:");
        board.print();
        score_board.print();
        let (x, y) = to_add;
        change_board(
            &mut board,
            &mut score_board,
            x,
            y,
            actual_player,
            // &table,
            &mut zhash,
        );
        println!("// final configuration:");
        board.print();
        score_board.print();
        false
    }

    #[test]
    fn test_add_board_00() {
        let black_pos = vec![(9, 8), (9, 7)];
        let white_pos = vec![];
        let to_add = (9, 6);
        let actual_player = Some(false);
        assert!(!test_add_board(black_pos, white_pos, to_add, actual_player));
    }

    #[test]
    fn test_add_board_01() {
        let black_pos = vec![(9, 8), (9, 7)];
        let white_pos = vec![];
        let to_add = (9, 6);
        let actual_player = Some(true);
        assert!(!test_add_board(black_pos, white_pos, to_add, actual_player));
    }

    #[test]
    fn test_add_board_02() {
        let black_pos = vec![(9, 8), (9, 7), (6, 6)];
        let white_pos = vec![(8, 6), (7, 6)];
        let to_add = (9, 6);
        let actual_player = Some(true);
        assert!(!test_add_board(black_pos, white_pos, to_add, actual_player));
    }

    fn test_remove_board(
        white_pos: Vec<(usize, usize)>,
        black_pos: Vec<(usize, usize)>,
        to_remove: (usize, usize),
        actual_player: Option<bool>,
        removed: Vec<((isize, isize), (isize, isize))>,
    ) -> bool {
        // let table = [[[0u64; 2]; SIZE_BOARD]; SIZE_BOARD];
        let mut zhash = 0u64;
        let mut bboard = [[None; SIZE_BOARD]; SIZE_BOARD];
        white_pos
            .iter()
            .for_each(|&(x, y)| bboard[x][y] = Some(true));
        black_pos
            .iter()
            .for_each(|&(x, y)| bboard[x][y] = Some(false));
        let mut board: Board = bboard.into();
        zobrist::init_zboard();
        let mut score_board = heuristic::evaluate_board(&mut board);
        println!("// Initial configuration:");
        board.print();
        score_board.print();
        let (x, y) = to_remove;
        remove_last_pawn(
            &mut board,
            &mut score_board,
            x,
            y,
            actual_player,
            removed,
            // &table,
            &mut zhash,
        );
        println!("// final configuration:");
        board.print();
        score_board.print();
        false
    }

    #[test]
    fn test_remove_board_00() {
        let black_pos = vec![(9, 8), (9, 7), (9, 6)];
        let white_pos = vec![(8, 6), (7, 6), (12, 6), (11, 7)];
        let to_remove = (9, 6);
        let actual_player = Some(true);
        let removed = vec![((10, 6), (11, 6))];
        assert!(!test_remove_board(
            black_pos,
            white_pos,
            to_remove,
            actual_player,
            removed
        ));
    }

    #[test]
    fn test_remove_board_01() {
        let black_pos = vec![(2, 0), (2, 1), (1, 0)];
        let white_pos = vec![(2, 2), (3, 1), (4, 0)];
        let to_remove = (2, 2);
        let actual_player = Some(false);
        let removed = vec![((1, 1), (0, 0))];
        assert!(!test_remove_board(
            black_pos,
            white_pos,
            to_remove,
            actual_player,
            removed
        ));
    }

    #[test]
    fn test_remove_board_02() {
        let black_pos = vec![(2, 0), (2, 1), (2, 2), (2, 3), (2, 4)];
        let white_pos = vec![];
        let to_remove = (2, 2);
        let actual_player = Some(false);
        let removed = vec![];
        assert!(!test_remove_board(
            black_pos,
            white_pos,
            to_remove,
            actual_player,
            removed
        ));
    }

    #[test]
    fn test_remove_board_03() {
        let black_pos = vec![(2, 0), (2, 1), (2, 2), (2, 3), (2, 4), (5, 5)];
        let white_pos = vec![(0, 2), (1, 2), (3, 2), (4, 2), (3, 3), (4, 4)];
        let to_remove = (2, 2);
        let actual_player = Some(false);
        let removed = vec![((0, 0), (1, 1))];
        assert!(!test_remove_board(
            black_pos,
            white_pos,
            to_remove,
            actual_player,
            removed
        ));
    }

    // fn test_change_board_hint(
    //     white_pos: Vec<(usize, usize)>,
    //     black_pos: Vec<(usize, usize)>,
    //     to_remove: (usize, usize),
    //     actual_player: Option<bool>,
    //     removed: Vec<((isize, isize), (isize, isize))>,
    // ) -> bool {
    //     let mut bboard = [[None; SIZE_BOARD]; SIZE_BOARD];
    //     white_pos
    //         .iter()
    //         .for_each(|&(x, y)| bboard[x][y] = Some(true));
    //     black_pos
    //         .iter()
    //         .for_each(|&(x, y)| bboard[x][y] = Some(false));
    //     let mut board: Board = bboard.into();
    //     let mut score_board = heuristic::evaluate_board(&mut board);
    //     println!("// Initial configuration:");
    //     board.print();
    //     score_board.print();
    //     let (x, y) = to_remove;
    //     let res = change_board_hint(&mut board, &mut score_board, x, y, actual_player);
    //     println!("// final configuration:");
    //     board.print();
    //     score_board.print();
    //     if removed.len() != res.len() {
    //         return false;
    //     }
    //     for i in 0..removed.len() {
    //         let ((x1, y1), (x2, y2)) = removed[i];
    //         let ((a1, b1), (a2, b2)) = removed[i];
    //         if x1 != a1 || x2 != a2 || y1 != b1 || y2 != b2 {
    //             return false;
    //         }
    //     }
    //     true
    // }

    // #[test]
    // fn test_change_board_hint_00() {
    //     let black_pos = vec![(2, 0), (2, 1), (2, 3), (2, 4), (5, 5)];
    //     let white_pos = vec![(0, 2), (1, 2), (3, 2), (4, 2), (3, 3), (4, 4)];
    //     let to_remove = (2, 2);
    //     let actual_player = Some(false);
    //     let removed = vec![((0, 0), (1, 1))];
    //     assert!(!test_change_board_hint(
    //         black_pos,
    //         white_pos,
    //         to_remove,
    //         actual_player,
    //         removed
    //     ));
    // }
}
