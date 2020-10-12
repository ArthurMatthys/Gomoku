use super::super::checks::after_turn_check::DIRECTIONS;
use super::super::checks::capture::DIRS;
use super::super::checks::double_three::check_double_three_hint;
use super::super::render::board::SIZE_BOARD;

use super::heuristic;
use super::threat_space::capture_coordinates_vec;
use super::threat_space::threat_search_space;
use super::threat_space::TypeOfThreat;
use super::zobrist;

const SCORE_ALIGN: i64 = 100;
const SCORE_TAKE: i64 = 100;

macro_rules! valid_coord {
    ($x: expr, $y: expr) => {
        $x >= 0 && $x < SIZE_BOARD as isize && $y >= 0 && $y < SIZE_BOARD as isize
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

//macro_rules! get_bool {
//    ($e:expr) => {
//        match $e {
//            Some(true) => "T",
//            Some(false) => "F",
//            None => "N",
//        }
//    };
//}
//macro_rules! index_edge {
//    ($delta:expr) => {
//        match delta {
//            (1, 1) => 1,
//            (1, 0) => 1,
//            (1, -1) => 1,
//            (0, 1) => 1,
//            (0, -1) => 0,
//            (-1, 1) => 0,
//            (-1, 0) => 0,
//            (-1, -1) => 0,
//        }
//    };
//}

//fn get_edge_case(
//    board: &[[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
//    actual: &Option<bool>,
//    x: &isize,
//    y: &isize,
//) -> isize {
//    if valid_coord!(*x, *y) {
//        match board[*x as usize][*y as usize] {
//            None => 1,
//            a if a == *actual => 2,
//            a if a != *actual => 3,
//        }
//    } else {
//        0
//    }
//}

fn print_board(board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD]) {
    for i in 0..19 {
        print!("// ");
        for j in 0..19 {
            match board[j][i] {
                Some(true) => print!("⊖"),
                Some(false) => print!("⊕"),
                None => print!("_"),
            }
        }
        println!();
    }
}
fn print_board_new_add(board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD], x: usize, y: usize) {
    for i in 0..19 {
        print!("// ");
        for j in 0..19 {
            if i == y && j == x {
                print!("⊛");
                continue;
            }
            match board[j][i] {
                Some(true) => print!("⊖"),
                Some(false) => print!("⊕"),
                None => print!("_"),
            }
        }
        println!();
    }
}
fn print_score_board(
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
) {
    for i in 0..19 {
        print!("// ");
        for j in 0..19 {
            for dir in 0..4 {
                match score_board[j][i][dir].1 {
                    Some(true) => print!("#"),
                    None => print!("|"),
                    Some(false) => print!("_"),
                }
                print!("{}", score_board[j][i][dir].0);
                match score_board[j][i][dir].2 {
                    Some(true) => print!("#"),
                    None => print!("|"),
                    Some(false) => print!("_"),
                }
            }
            print!(" ");
        }
        println!();
    }
}

// Modify
fn modify_align(
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    (x, y): (isize, isize),
    (dx, dy): (&isize, &isize),
    dir: usize,
    len_change: u8,
    len_align: u8,
    left_edge: Option<bool>,
    right_edge: Option<bool>,
) {
    let mut new_x = x;
    let mut new_y = y;
    for _ in 0..len_change {
        new_x += dx;
        new_y += dy;
        if !valid_coord!(new_x, new_y) {
            print_score_board(score_board);
            println!(
                "start : ({},{})\ndirection : ({},{})\nlen_to_change : {}\nlen_to_put : {}",
                x, y, dx, dy, len_change, len_align
            );
        }
        score_board[new_x as usize][new_y as usize][dir] = (len_align, left_edge, right_edge);
    }
}

fn decrease_align(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
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
        //        println!(
        //            "({}/{})=> pawn_ref : {}//pawn: {}",
        //            new_x,
        //            new_y,
        //            get_bool!(pawn),
        //            get_bool!(board[new_x as usize][new_y as usize]),
        //        );
        if !valid_coord!(new_x, new_y) || board[new_x as usize][new_y as usize] != pawn {
            //            println!("stop {}", nbr_align);
            for &(old_x, old_y) in to_change.iter() {
                //                println!("change here");
                score_board[old_x as usize][old_y as usize][dir] =
                    (nbr_align, left_edge, right_edge)
            }
            break;
        } else {
            to_change.push((new_x, new_y));
            nbr_align += 1;
        }
    }
}

// Change score board after the removal of a pawn
fn change_score_board_remove(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    x: isize,
    y: isize,
) {
    //    println!("remove");
    let pawn = board[x as usize][y as usize];
    board[x as usize][y as usize] = None;
    if pawn == None {
        println!("Pas content");
        unreachable!();
    }
    for (i, (dx, dy)) in DIRECTIONS.iter().enumerate() {
        score_board[x as usize][y as usize][i] = (0, Some(false), Some(false));
        for way in [-1, 1].iter() {
            let new_x = x + (way * dx);
            let new_y = y + (way * dy);
            if valid_coord!(new_x, new_y) {
                match board[new_x as usize][new_y as usize] {
                    None => (),
                    //                   None => (println!("Hello3")),
                    a if a == pawn => {
                        let (_, mut left_edge, mut right_edge) =
                            score_board[new_x as usize][new_y as usize][i];
                        if *way == -1 {
                            right_edge = Some(false);
                        } else {
                            left_edge = Some(false);
                        }
                        //                        println!("remove decrease");
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
                    a if a != pawn => {
                        if a == None {}
                        let (align, mut left_edge, mut right_edge) =
                            score_board[new_x as usize][new_y as usize][i];
                        if *way == -1 {
                            right_edge = Some(false);
                        } else {
                            left_edge = Some(false);
                        }
                        if align >= 7 {
                            println!("wtf2");
                        }
                        //                        println!("remove modify");
                        modify_align(
                            score_board,
                            (x, y),
                            (&(way * dx), &(way * dy)),
                            i,
                            align,
                            align,
                            left_edge,
                            right_edge,
                        );
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}

use std::process;

// Change score_board when add a pawn
pub fn change_score_board_add(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    x: isize,
    y: isize,
    pawn: Option<bool>,
) {
    //    println!("add");
    if board[x as usize][y as usize] != None {
        println!("wtf");
        unreachable!();
    }
    board[x as usize][y as usize] = pawn;
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
        let x_left = x - dx;
        let y_left = y - dy;

        // Get case on the left hand side of the align
        // (board edge, empty slot, ally pawnm ennemy pawn)
        let edge_case_left = if !valid_coord!(x_left, y_left) {
            0
        } else {
            get_edge_case(&pawn, board[x_left as usize][y_left as usize])
        };
        let (mut align_left, mut left_edge_left): (u8, Option<bool>) = (0, None);
        match edge_case_left {
            // edge
            0 => (),
            // empty slot
            1 => left_edge_left = Some(false),
            // align ally
            2 => {
                let focused_tuple = score_board[x_left as usize][y_left as usize][i];
                align_left = focused_tuple.0;
                left_edge_left = focused_tuple.1;
                tot_align += align_left;
            }
            // ennemy align
            3 => {
                align_left = score_board[x_left as usize][y_left as usize][i].0;
                left_edge_left = Some(true);
            }
            _ => unreachable!(),
        };

        let x_right = x + dx;
        let y_right = y + dy;
        // Get case on the right hand side of the align
        // (board edge, empty slot, ally pawnm ennemy pawn)
        let edge_case_right = if !valid_coord!(x_right, y_right) {
            0
        } else {
            get_edge_case(&pawn, board[x_right as usize][y_right as usize])
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
                let focused_tuple = score_board[x_right as usize][y_right as usize][i];
                align_right = focused_tuple.0;
                right_edge_right = focused_tuple.2;
                tot_align += align_right;
            }
            // ennemy align
            3 => {
                align_right = score_board[x_right as usize][y_right as usize][i].0;
                right_edge_right = Some(true);
            }
            _ => unreachable!(),
        };
        //        println!(
        //            "left : {}->{}//right: {}->{}",
        //            edge_case_left,
        //            get_bool!(left_edge_left),
        //            edge_case_right,
        //            get_bool!(right_edge_right)
        //        );

        //        let mut count = 0;
        //        for line in 0..19 {
        //            for col in 0..19 {
        //                if board[col][line] != None {
        //                    count += 1;
        //                }
        //            }
        //        }
        //        if count >= 56 {
        //            println!("hello2");
        //            for line in 0..19 {
        //                print!("// ");
        //                for col in 0..19 {
        //                    match board[col][line] {
        //                        Some(true) => print!("⊖"),
        //                        Some(false) => print!("⊕"),
        //                        None => print!("_"),
        //                    }
        //                }
        //                println!();
        //            }
        //            println!("remove modify (arg :)\n score_board :\n");
        //            for line in 0..19 {
        //                for col in 0..19 {
        //                    for dir in 0..4 {
        //                        print!("{:2}", score_board[col][line][dir].0);
        //                    }
        //                    print!(" ");
        //                }
        //                println!();
        //            }
        //            println!(
        //                "Coord : ({},{})\nDelta : ({},{})\nDir : {}\nNbr to change : {}\nLen align: {}",
        //                x, y, dx, dy, i, align_left, tot_align,
        //            );
        //        }
        //        println!("Crash left");
        //        if align_left >= 7 {
        //            println!("wtf_left");
        //        }
        //        if align_right >= 7 {
        //            println!("wtf_right");
        //        }
        score_board[x as usize][y as usize][i] = (tot_align, left_edge_left, right_edge_right);
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
                score_board[x_left as usize][y_left as usize][i].1,
                Some(true),
            ),
            0..=1 => (),
            _ => unreachable!(),
        }
        //        println!("Crash right");
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
                score_board[x_right as usize][y_right as usize][i].2,
            ),
            0..=1 => (),
            _ => unreachable!(),
        }
        //        if tot_align >= 7 {
        //            println!("wtf_tot");
        //            print_board(board);
        //            print_score_board(score_board);
        //            println!(
        //                "start : ({},{})\ndirection : ({},{})\nlen_to_put : {}\ncase_left : {}\ncase_right : {}",
        //                x, y, dx, dy, tot_align, edge_case_left, edge_case_right
        //            );
        //        }
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

pub fn get_space(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    actual_player: Option<bool>,
    actual_take: isize,
) -> Vec<(usize, usize, i64)> {
    let mut ret = Vec::with_capacity(200);
    for x in 0..SIZE_BOARD {
        for y in 0..SIZE_BOARD {
            let value = board[x][y];
            if value == None {
                let mut check = 0;
                let mut to_add = false;
                let mut score = 0i64;
                for &(dx, dy) in DIRS.iter() {
                    let new_x = x as isize + dx;
                    let new_y = y as isize + dy;
                    if valid_coord!(new_x, new_y) && board[new_x as usize][new_y as usize] != None {
                        if check != 0
                            || !check_double_three_hint(
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
                            if valid_coord!(opp_x, opp_y) {
                                if board[opp_x as usize][opp_y as usize] == value {
                                    let opp_tuple =
                                        score_board[opp_x as usize][opp_y as usize][dir];
                                    opp_align = opp_tuple.0;
                                    edge_opp = get_other_edge!(opp_tuple, (way + 1) % 2);
                                    to_add = true;
                                } else if board[opp_x as usize][opp_y as usize] == None {
                                    edge_opp = Some(false);
                                } else {
                                    edge_opp = Some(true);
                                    to_add = true;
                                }
                            }
                            check = 1;
                            let tuple_focused = score_board[new_x as usize][new_y as usize][dir];
                            score += get_score_board(
                                tuple_focused.0,
                                actual_player == board[new_x as usize][new_y as usize],
                                get_other_edge!(tuple_focused, way),
                                actual_take,
                                opp_align,
                                edge_opp,
                            );
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

pub fn find_available_pos(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    player_actual: Option<bool>,
) -> Vec<(usize, usize)> {
    let mut ret: Vec<(usize, usize)> = vec![];
    for x in 0..SIZE_BOARD {
        for y in 0..SIZE_BOARD {
            if board[x][y] == None {
                for &(dx, dy) in DIRS.iter() {
                    let new_x = x as isize + dx;
                    let new_y = y as isize + dy;
                    if valid_coord!(new_x, new_y) && board[new_x as usize][new_y as usize] != None {
                        if !check_double_three_hint(board, player_actual, x as isize, y as isize) {
                            ret.push((x as usize, y as usize));
                        }
                        break;
                    }
                }
            }
        }
    }
    ret
}

pub fn find_continuous_threats(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    player_actual: Option<bool>,
    player_actual_catch: &mut isize,
    player_opposite_catch: &mut isize,
    depth: &mut i8,
    depth_win: &mut i8,
    true_actual_player: bool,
) -> Option<(usize, usize)> {
    //    println!("Start FCT");
    //    print_board(board);
    //    println!("----");
    if *depth < *depth_win {
        return None;
    }
    let threats: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
        threat_search_space(board, score_board, player_actual, player_actual_catch);

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

    for (threat, _, counters) in threats.iter() {
        let (x, y) = threat;
        //        println!("CT_threat");
        if board[*x][*y] != None {
            println!("double wtf");
            unreachable!();
        }
        let removed = change_board_hint(board, score_board, *x, *y, player_actual);
        *player_actual_catch += removed.len() as isize;
        //        print_board(board);
        //        println!("----");

        let mut counters_valid: Vec<(usize, usize)> = vec![];
        for (opp_x, opp_y) in counters.iter() {
            if !check_double_three_hint(
                board,
                get_opp!(player_actual),
                *opp_x as isize,
                *opp_y as isize,
            ) {
                counters_valid.push((*opp_x, *opp_y));
            }
        }
        //        print_board(board);
        //        println!("----");

        if counters_valid.len() == 0 {
            *depth_win = *depth;
            //            println!("NAN MAIS WSH");
            remove_last_pawn_hint(board, score_board, *x, *y, player_actual, removed);
            return Some((*x, *y));
        }

        let mut win: bool = true;

        //        println!("iter counters");
        for (counter_x, counter_y) in counters_valid.iter() {
            //            println!("CT_counter");
            if board[*counter_x][*counter_y] != None {
                //                print_board_new_add(board, *x, *y);
                //                println!(
                //                    "actual : {}\ncounters : ({},{})\nthreat : ({},{})\ntype of threat : {}",
                //                    match player_actual {
                //                        None => unreachable!(),
                //                        Some(true) => "+",
                //                        Some(false) => "-",
                //                    },
                //                    *counter_x,
                //                    *counter_y,
                //                    *x,
                //                    *y,
                //                    match type_o {
                //                        TypeOfThreat::FiveTake => "FiveTake",
                //                        TypeOfThreat::FourTake => "FourTake",
                //                        TypeOfThreat::ThreeTake => "ThreeTake",
                //                        TypeOfThreat::TwoTake => "TwoTake",
                //                        TypeOfThreat::OneTake => "OneTake",
                //                        TypeOfThreat::FourOC => "FourOC",
                //                        TypeOfThreat::FourOF => "FourOF",
                //                        TypeOfThreat::FourSOC => "FourSOC",
                //                        TypeOfThreat::FourSOF => "FourSOF",
                //                        TypeOfThreat::ThreeOC => "ThreeOC",
                //                        TypeOfThreat::ThreeOF => "ThreeOF",
                //                        TypeOfThreat::WIN => "WIN",
                //                        TypeOfThreat::AlreadyWon => "WON",
                //                        TypeOfThreat::EMPTY => "EMPTY",
                //                    }
                //                );
                //                println!("got it");
                unreachable!();
            }
            let removed_counter = change_board_hint(
                board,
                score_board,
                *counter_x,
                *counter_y,
                get_opp!(player_actual),
            );

            *player_opposite_catch += removed_counter.len() as isize;
            let res = find_continuous_threats(
                board,
                score_board,
                player_actual,
                player_actual_catch,
                player_opposite_catch,
                &mut (*depth - 2),
                depth_win,
                true_actual_player,
            );
            *player_opposite_catch -= removed_counter.len() as isize;
            remove_last_pawn_hint(
                board,
                score_board,
                *counter_x,
                *counter_y,
                get_opp!(player_actual),
                removed_counter,
            );

            if res == None {
                win = false;
            }
        }
        *player_actual_catch -= removed.len() as isize;
        remove_last_pawn_hint(board, score_board, *x, *y, player_actual, removed);

        if win {
            *depth_win = *depth;
            return Some((*x, *y));
        }
    }
    None
}

macro_rules! explore_align_light {
    (
        $board: expr,
        $new_line: expr,
        $new_col: expr,
        $actual_player: expr,
        $dir: expr,
        $orientation: expr
    ) => {
        while valid_coord!($new_line, $new_col)
            && $board[$new_line as usize][$new_col as usize] == $actual_player
        {
            $new_line += (DIRECTIONS[$dir].0 * $orientation);
            $new_col += (DIRECTIONS[$dir].1 * $orientation);
        }
    };
}

//TODO
fn best_of_board(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    player_actual: Option<bool>,
    lst_moove: Vec<Option<(usize, usize)>>,
) -> Option<(usize, usize)> {
    None
}

pub fn null_move_heuristic(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    player_actual: Option<bool>,
    player_actual_catch: &mut isize,
    player_opposite_catch: &mut isize,
) -> Option<(usize, usize)> {
    let actual_threat = threat_search_space(board, score_board, player_actual, player_actual_catch);
    let opp_threat = threat_search_space(
        board,
        score_board,
        get_opp!(player_actual),
        player_opposite_catch,
    );
    if opp_threat.len() == 0 || opp_threat[0].1 < TypeOfThreat::FourOF {
        return None;
    } else if opp_threat[0].1 == TypeOfThreat::AlreadyWon {
        for line in 0..19 {
            for col in 0..19 {
                for dir in 0..4 {
                    if score_board[line][col][dir].0 >= 5 {
                        let len = score_board[line][col][dir].0 as isize;
                        let mut new_x = line as isize;
                        let mut new_y = col as isize;
                        let (dx, dy) = DIRS[dir];
                        explore_align_light!(board, new_x, new_y, get_opp!(player_actual), dir, -1);
                        let mut to_take: Vec<(usize, usize)> = vec![];
                        let start: isize = len - 5;
                        let end: isize = start + (10 - len);
                        for step in start..end {
                            to_take.push((
                                (new_x + dx * (step + 1)) as usize,
                                (new_y + dy * (step + 1)) as usize,
                            ));
                        }
                        let captures = capture_coordinates_vec(
                            score_board,
                            board,
                            get_opp!(player_actual),
                            to_take,
                            dir,
                        );
                        //TODO ^ add filter double three
                        if captures.len() > 0 {
                            return Some(captures[0]);
                        } else {
                            return None;
                        }
                    }
                }
            }
        }
        return None;
    } else if actual_threat.len() == 0 {
        return Some(opp_threat[0].0);
    } else {
        if actual_threat[0].1 >= opp_threat[0].1 {
            return None;
        } else if opp_threat[0].1 >= TypeOfThreat::FourOF {
            return Some(opp_threat[0].0);
        } else {
            return None;
        }
    }
}

//macro_rules! get_bool {
//    ($e:expr) => {
//        match $e {
//            Some(true) => "T",
//            Some(false) => "F",
//            None => "N",
//        }
//    };
//}

pub fn board_state_win(
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
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
                let focused_tuple = score_board[x][y][dir];
                if winner_align || focused_tuple.0 >= 5 {
                    len_align = focused_tuple.0;
                    dir_align = dir;
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
                let mut win = false;
                let mut new_x = x as isize;
                let mut new_y = y as isize;
                let direction = DIRS[dir_align];
                for way in [-1, 1].iter() {
                    loop {
                        new_x += way * direction.0;
                        new_y += way * direction.1;
                        if valid_coord!(new_x, new_y)
                            && score_board[new_x as usize][new_y as usize][dir_align].0 == len_align
                        {
                            for dir in 0..4 {
                                if dir == dir_align {
                                    continue;
                                }
                                let focused_tuple =
                                    score_board[new_x as usize][new_y as usize][dir];
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
                        } else {
                            break;
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
    ($table:expr, $zhash:expr, $x:expr, $y:expr, $piece:expr) => {
        *$zhash ^= $table[$x as usize][$y as usize][zobrist::ZPIECES[$piece]];
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

pub fn remove_last_pawn_hint(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    x: usize,
    y: usize,
    pawn: Option<bool>,
    removed: Vec<((isize, isize), (isize, isize))>,
) {
    let old = get_opp!(pawn);
    change_score_board_remove(board, score_board, x as isize, y as isize);
    removed.iter().for_each(|&((x1, y1), (x2, y2))| {
        //        println!("fixed removed_hint");
        change_score_board_add(board, score_board, x1 as isize, y1 as isize, old);
        change_score_board_add(board, score_board, x2 as isize, y2 as isize, old);
    })
}

pub fn change_board_hint(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    x: usize,
    y: usize,
    pawn: Option<bool>,
) -> Vec<((isize, isize), (isize, isize))> {
    let mut removed = Vec::with_capacity(16);
    //    println!("add pawn_hint");
    change_score_board_add(board, score_board, x as isize, y as isize, pawn);
    let opp = get_opp!(pawn);
    for &(dx, dy) in DIRS.iter() {
        let mut count = 0;
        let mut new_x = x as isize;
        let mut new_y = y as isize;
        for _ in 0..3 {
            new_x += dx;
            new_y += dy;
            if !valid_coord!(new_x, new_y) {
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
            change_score_board_remove(board, score_board, x1 as isize, y1 as isize);
            change_score_board_remove(board, score_board, x2 as isize, y2 as isize);
            removed.push(((x1, y1), (x2, y2)));
        }
    }

    removed
}

pub fn remove_last_pawn(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    x: usize,
    y: usize,
    pawn: Option<bool>,
    removed: Vec<((isize, isize), (isize, isize))>,
    table: &[[[u64; 2]; SIZE_BOARD]; SIZE_BOARD],
    zhash: &mut u64,
) {
    let old = get_opp!(pawn);
    change_score_board_remove(board, score_board, x as isize, y as isize);
    add_zhash!(table, zhash, x, y, get_zindex_from_pawn!(pawn));
    removed.iter().for_each(|&((x1, y1), (x2, y2))| {
        //        println!("fixed removed");
        add_zhash!(table, zhash, x1, y1, get_zindex_from_pawn!(old));
        change_score_board_add(board, score_board, x1 as isize, y1 as isize, old);
        add_zhash!(table, zhash, x2, y2, get_zindex_from_pawn!(old));
        change_score_board_add(board, score_board, x2 as isize, y2 as isize, old);
    })
}

pub fn change_board(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    x: usize,
    y: usize,
    pawn: Option<bool>,
    table: &[[[u64; 2]; SIZE_BOARD]; SIZE_BOARD],
    zhash: &mut u64,
) -> Vec<((isize, isize), (isize, isize))> {
    let mut removed = Vec::with_capacity(16);
    //    println!("add pawn");
    change_score_board_add(board, score_board, x as isize, y as isize, pawn);
    add_zhash!(table, zhash, x, y, get_zindex_from_pawn!(pawn));
    let opp = get_opp!(pawn);
    for &(dx, dy) in DIRS.iter() {
        let mut count = 0;
        let mut new_x = x as isize;
        let mut new_y = y as isize;
        for _ in 0..3 {
            new_x += dx;
            new_y += dy;
            if !valid_coord!(new_x, new_y) {
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
            change_score_board_remove(board, score_board, x1 as isize, y1 as isize);
            add_zhash!(table, zhash, x1, y1, get_zindex_from_pawn!(opp));
            change_score_board_remove(board, score_board, x2 as isize, y2 as isize);
            add_zhash!(table, zhash, x2, y2, get_zindex_from_pawn!(opp));
            removed.push(((x1, y1), (x2, y2)));
        }
    }

    removed
}
#[cfg(test)]
mod tests {
    use super::*;

    pub fn find_continuous_threats_print(
        board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
        score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
        player_actual: Option<bool>,
        player_actual_catch: &mut isize,
        player_opposite_catch: &mut isize,
        depth: &mut i8,
        depth_win: &mut i8,
    ) -> Option<(usize, usize)> {
        if *depth < *depth_win {
            return None;
        }
        let threats: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            threat_search_space(board, score_board, player_actual, player_actual_catch);

        println!("hello : {} threat", threats.len());
        if threats.len() == 0 {
            return None;
        }

        if threats.len() == 1 && threats[0].1 == TypeOfThreat::WIN {
            *depth_win = *depth;
            let (x, y) = threats[0].0;
            println!("Winner threat in ({},{})", x, y);
            for i in 0..19 {
                for j in 0..19 {
                    if i == y && j == x {
                        print!("⊛")
                    } else {
                        match board[j][i] {
                            Some(true) => print!("⊖"),
                            Some(false) => print!("⊕"),
                            None => print!("_"),
                        }
                    }
                }
                println!();
            }
            return Some(threats[0].0);
        }

        for (threat, typeofthreat, counters) in threats.iter() {
            let (x, y) = threat;
            let removed = change_board_hint(board, score_board, *x, *y, player_actual);
            *player_actual_catch += removed.len() as isize;

            println!(
                "Config depth {}, black pawn in ({},{}), threat : {} ",
                depth,
                x,
                y,
                match typeofthreat {
                    TypeOfThreat::FiveTake => "FiveTake",
                    TypeOfThreat::FourTake => "FourTake",
                    TypeOfThreat::ThreeTake => "ThreeTake",
                    TypeOfThreat::TwoTake => "TwoTake",
                    TypeOfThreat::OneTake => "OneTake",
                    TypeOfThreat::FourOC => "FourOC",
                    TypeOfThreat::FourOF => "FourOF",
                    TypeOfThreat::FourSOC => "FourSOC",
                    TypeOfThreat::FourSOF => "FourSOF",
                    TypeOfThreat::ThreeOC => "ThreeOC",
                    TypeOfThreat::ThreeOF => "ThreeOF",
                    TypeOfThreat::WIN => "WIN",
                    TypeOfThreat::AlreadyWon => "WON",
                    TypeOfThreat::EMPTY => "EMPTY",
                }
            );
            for i in 0..19 {
                for j in 0..19 {
                    if i == *y && j == *x {
                        print!("⊛")
                    } else {
                        match board[j][i] {
                            Some(true) => print!("⊖"),
                            Some(false) => print!("⊕"),
                            None => print!("_"),
                        }
                    }
                }
                println!();
            }
            let mut counters_valid: Vec<(usize, usize)> = vec![];
            for (opp_x, opp_y) in counters.iter() {
                if !check_double_three_hint(
                    board,
                    get_opp!(player_actual),
                    *opp_x as isize,
                    *opp_y as isize,
                ) {
                    counters_valid.push((*opp_x, *opp_y));
                }
            }
            print!("Counters : ");
            counters_valid
                .iter()
                .for_each(|(c_x, c_y)| print!("({},{}); ", c_x, c_y));
            println!();

            if counters_valid.len() == 0 && *typeofthreat < TypeOfThreat::FiveTake {
                for (x, y) in find_available_pos(board, get_opp!(player_actual)) {
                    if !counters_valid
                        .iter()
                        .any(|&(cmp_x, cmp_y)| cmp_x == x && cmp_y == y)
                    {
                        counters_valid.push((x, y));
                    }
                }
            }

            print!("Counters : ");
            counters_valid
                .iter()
                .for_each(|(c_x, c_y)| print!("({},{}); ", c_x, c_y));
            println!();

            let mut win: bool = true;

            for (counter_x, counter_y) in counters_valid.iter() {
                let removed_counter = change_board_hint(
                    board,
                    score_board,
                    *counter_x,
                    *counter_y,
                    get_opp!(player_actual),
                );
                println!(
                    "Config depth {}, white pawn in ({},{}) ",
                    depth, counter_x, counter_y
                );
                for i in 0..19 {
                    for j in 0..19 {
                        if i == *counter_y && j == *counter_x {
                            print!("⊙")
                        } else {
                            match board[j][i] {
                                Some(true) => print!("⊖"),
                                Some(false) => print!("⊕"),
                                None => print!("_"),
                            }
                        }
                    }
                    println!();
                }

                *player_opposite_catch += removed_counter.len() as isize;
                let res = find_continuous_threats(
                    board,
                    score_board,
                    player_actual,
                    player_actual_catch,
                    player_opposite_catch,
                    &mut (*depth - 2),
                    depth_win,
                    true,
                );
                *player_opposite_catch -= removed_counter.len() as isize;
                remove_last_pawn_hint(
                    board,
                    score_board,
                    *counter_x,
                    *counter_y,
                    get_opp!(player_actual),
                    removed_counter,
                );
                println!();
                println!();

                if res == None {
                    win = false;
                }
            }
            *player_actual_catch -= removed.len() as isize;
            remove_last_pawn_hint(board, score_board, *x, *y, player_actual, removed);

            if win {
                println!("Winner threat in ({},{})", *x, *y);
                for i in 0..19 {
                    for j in 0..19 {
                        if i == *y && j == *x {
                            print!("⊛")
                        } else {
                            match board[j][i] {
                                Some(true) => print!("⊖"),
                                Some(false) => print!("⊕"),
                                None => print!("_"),
                            }
                        }
                    }
                    println!();
                }
                *depth_win = *depth;
                return Some((*x, *y));
            }
        }
        None
    }

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
        let mut test_board = [[None; SIZE_BOARD]; SIZE_BOARD];
        let mut score_tab: [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD] =
            [[[(0, Some(false), Some(false)); 4]; SIZE_BOARD]; SIZE_BOARD];
        white_pos.iter().for_each(|&(x, y)| {
            test_board[x][y] = Some(true);
            change_score_board_add(&mut test_board, &mut score_tab, x as isize, y as isize);
        });
        black_pos.iter().for_each(|&(x, y)| {
            test_board[x][y] = Some(false);
            change_score_board_add(&mut test_board, &mut score_tab, x as isize, y as isize);
        });
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
        for i in 0..19 {
            for j in 0..19 {
                match test_board[j][i] {
                    Some(true) => print!("B"),
                    Some(false) => print!("N"),
                    None => print!("E"),
                }
                score_tab[j][i].iter().for_each(|&(value, a, b)| {
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

    //    #[test]
    //    fn test_add_pawn_scoreboard0() {
    //        let history_pos = vec![
    //            (8, 8),
    //            (7, 7),
    //            (8, 9),
    //            (9, 9),
    //            (8, 10),
    //            (8, 11),
    //            (8, 7),
    //            (8, 6),
    //            (7, 7),
    //            (9, 10),
    //            (5, 5),
    //            (4, 4),
    //            (6, 6),
    //        ];
    //        let history_remove = vec![(8, 8)];
    //        assert!(test_score_board(history_pos, history_remove))
    //    }

    // fn test_score_board(
    //     history_pos: Vec<(usize, usize)>,
    //     history_remove: Vec<(usize, usize)>,
    // ) -> bool {
    //     let mut test_board = [[None; SIZE_BOARD]; SIZE_BOARD];
    //     let mut score_board: [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD] =
    //         [[[(0, Some(false), Some(false)); 4]; SIZE_BOARD]; SIZE_BOARD];
    //     let mut pawn = Some(false);

    //     for &(x, y) in history_pos.iter() {
    //         test_board[x][y] = pawn;
    //         change_score_board_add(&mut test_board, &mut score_board, x as isize, y as isize);
    //         for i in 0..19 {
    //             for j in 0..19 {
    //                 match test_board[j][i] {
    //                     Some(true) => print!("⊖"),
    //                     Some(false) => print!("⊕"),
    //                     None => print!("_"),
    //                 }
    //             }
    //             println!();
    //         }
    //         for i in 0..19 {
    //             for j in 0..19 {
    //                 match test_board[j][i] {
    //                     Some(true) => print!("W"),
    //                     Some(false) => print!("B"),
    //                     None => print!("E"),
    //                 }
    //                 score_board[j][i].iter().for_each(|&(value, a, b)| {
    //                     print!("{:2}{}{}", value, get_bool!(a), get_bool!(b))
    //                 });
    //                 print!(" ");
    //             }
    //             println!();
    //         }
    //         if pawn == Some(false) {
    //             pawn = Some(true);
    //         } else {
    //             pawn = Some(false);
    //         }
    //         println!("-----------");
    //     }
    //     for &(x, y) in history_remove.iter() {
    //         change_score_board_remove(&mut test_board, &mut score_board, x as isize, y as isize);
    //         test_board[x][y] = None;
    //         for i in 0..19 {
    //             for j in 0..19 {
    //                 match test_board[j][i] {
    //                     Some(true) => print!("⊖"),
    //                     Some(false) => print!("⊕"),
    //                     None => print!("_"),
    //                 }
    //             }
    //             println!();
    //         }
    //         for i in 0..19 {
    //             for j in 0..19 {
    //                 match test_board[j][i] {
    //                     Some(true) => print!("W"),
    //                     Some(false) => print!("B"),
    //                     None => print!("E"),
    //                 }
    //                 score_board[j][i].iter().for_each(|&(value, a, b)| {
    //                     print!("{:2}{}{}", value, get_bool!(a), get_bool!(b))
    //                 });
    //                 print!(" ");
    //             }
    //             println!();
    //         }
    //         if pawn == Some(false) {
    //             pawn = Some(true);
    //         } else {
    //             pawn = Some(false);
    //         }
    //         println!("-----------");
    //     }
    //     false
    // }

    fn test_get_space(
        white_pos: Vec<(usize, usize)>,
        black_pos: Vec<(usize, usize)>,
        actual_take: &mut isize,
        actual_player: Option<bool>,
        expected_result: Vec<(usize, usize, i64)>,
    ) -> bool {
        let mut test_board_tmp = [[None; SIZE_BOARD]; SIZE_BOARD];
        let mut test_board = [[None; SIZE_BOARD]; SIZE_BOARD];
        white_pos
            .iter()
            .for_each(|&(x, y)| test_board_tmp[x][y] = Some(1));
        black_pos
            .iter()
            .for_each(|&(x, y)| test_board_tmp[x][y] = Some(0));

        white_pos
            .iter()
            .for_each(|&(x, y)| test_board[x][y] = Some(true));
        black_pos
            .iter()
            .for_each(|&(x, y)| test_board[x][y] = Some(false));

        // Print initial configuration
        println!("// Initial configuration:");
        for i in 0..19 {
            print!("// ");
            for j in 0..19 {
                match test_board[j][i] {
                    Some(true) => print!("⊖"),
                    Some(false) => print!("⊕"),
                    None => print!("_"),
                }
            }
            println!();
        }
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
        let mut test_board = [[None; SIZE_BOARD]; SIZE_BOARD];
        white_pos
            .iter()
            .for_each(|&(x, y)| test_board[x][y] = Some(true));
        black_pos
            .iter()
            .for_each(|&(x, y)| test_board[x][y] = Some(false));
        let mut score_board = heuristic::evaluate_board(&mut test_board);
        // Print initial configuration
        println!("// Initial configuration:");
        for i in 0..19 {
            print!("// ");
            for j in 0..19 {
                match test_board[j][i] {
                    Some(true) => print!("⊖"),
                    Some(false) => print!("⊕"),
                    None => print!("_"),
                }
            }
            println!();
        }
        let res = find_continuous_threats(
            &mut test_board,
            &mut score_board,
            actual_player,
            actual_take,
            opp_take,
            depth,
            depth_win,
            true,
        );
        match res {
            None => println!("No threat found"),
            Some((x, y)) => println!("Threat found {}:{}", x, y),
        };

        res == intented
    }

    fn test_continuous_threats_print(
        white_pos: Vec<(usize, usize)>,
        black_pos: Vec<(usize, usize)>,
        actual_player: Option<bool>,
        actual_take: &mut isize,
        opp_take: &mut isize,
        depth: &mut i8,
        depth_win: &mut i8,
        intented: Option<(usize, usize)>,
    ) -> bool {
        let mut test_board = [[None; SIZE_BOARD]; SIZE_BOARD];
        white_pos
            .iter()
            .for_each(|&(x, y)| test_board[x][y] = Some(true));
        black_pos
            .iter()
            .for_each(|&(x, y)| test_board[x][y] = Some(false));
        let mut score_board = heuristic::evaluate_board(&mut test_board);
        // Print initial configuration
        println!("// Initial configuration:");
        for i in 0..19 {
            print!("// ");
            for j in 0..19 {
                match test_board[j][i] {
                    Some(true) => print!("⊖"),
                    Some(false) => print!("⊕"),
                    None => print!("_"),
                }
            }
            println!();
        }
        let res = find_continuous_threats_print(
            &mut test_board,
            &mut score_board,
            actual_player,
            actual_take,
            opp_take,
            depth,
            depth_win,
        );
        match res {
            None => println!("No threat found"),
            Some((x, y)) => println!("Threat found {}:{}", x, y),
        };

        res == intented
    }

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

    //    #[test]
    //    fn test_threat_no_threat_02() {
    //        let black_pos = vec![(9, 8), (9, 7), (9, 6), (10, 6), (11, 7), (8, 5), (7, 5)];
    //        let white_pos = vec![(7, 3)];
    //        let mut white_take = 0_isize;
    //        let mut black_take = 0_isize;
    //        let mut depth = 2_i8;
    //        let mut depth_max = 0_i8;
    //        let expected_result = Some((0, 0));
    //
    //        assert!(test_continuous_threats_print(
    //            white_pos,
    //            black_pos,
    //            Some(false),
    //            &mut black_take,
    //            &mut white_take,
    //            &mut depth,
    //            &mut depth_max,
    //            expected_result
    //        ))
    //    }

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

    //    #[test]
    //    fn test_threat_board_00() {
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
    //        let mut white_take = 0_isize;
    //        let mut black_take = 0_isize;
    //        let mut depth = 12_i8;
    //        let mut depth_max = 0_i8;
    //        let expected_result = None;
    //
    //        assert!(test_continuous_threats_print(
    //            white_pos,
    //            black_pos,
    //            Some(false),
    //            &mut black_take,
    //            &mut white_take,
    //            &mut depth,
    //            &mut depth_max,
    //            expected_result
    //        ))
    //    }
    //pub fn change_score_board_add(
    //    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    //    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    //    x: isize,
    //    y: isize,
    //) {

    fn print_board(board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD]) {
        for i in 0..19 {
            print!("// ");
            for j in 0..19 {
                match board[j][i] {
                    Some(true) => print!("⊖"),
                    Some(false) => print!("⊕"),
                    None => print!("_"),
                }
            }
            println!();
        }
    }

    fn print_score_board(
        score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    ) {
        for i in 0..19 {
            print!("// ");
            for j in 0..19 {
                for dir in 0..4 {
                    match score_board[j][i][dir].1 {
                        Some(true) => print!("#"),
                        None => print!("|"),
                        Some(false) => print!("_"),
                    }
                    print!("{}", score_board[j][i][dir].0);
                    match score_board[j][i][dir].2 {
                        Some(true) => print!("#"),
                        None => print!("|"),
                        Some(false) => print!("_"),
                    }
                }
                print!(" ");
            }
            println!();
        }
    }

    fn test_add_board(
        white_pos: Vec<(usize, usize)>,
        black_pos: Vec<(usize, usize)>,
        to_add: (usize, usize),
        actual_player: Option<bool>,
    ) -> bool {
        let mut board = [[None; SIZE_BOARD]; SIZE_BOARD];
        let table = [[[0u64; 2]; SIZE_BOARD]; SIZE_BOARD];
        let mut zhash = 0u64;
        white_pos
            .iter()
            .for_each(|&(x, y)| board[x][y] = Some(true));
        black_pos
            .iter()
            .for_each(|&(x, y)| board[x][y] = Some(false));
        let mut score_board = heuristic::evaluate_board(&mut board);
        println!("// Initial configuration:");
        print_board(&mut board);
        print_score_board(&mut score_board);
        let (x, y) = to_add;
        change_board(
            &mut board,
            &mut score_board,
            x,
            y,
            actual_player,
            &table,
            &mut zhash,
        );
        println!("// final configuration:");
        print_board(&mut board);
        print_score_board(&mut score_board);
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
        let table = [[[0u64; 2]; SIZE_BOARD]; SIZE_BOARD];
        let mut zhash = 0u64;
        let mut board = [[None; SIZE_BOARD]; SIZE_BOARD];
        white_pos
            .iter()
            .for_each(|&(x, y)| board[x][y] = Some(true));
        black_pos
            .iter()
            .for_each(|&(x, y)| board[x][y] = Some(false));
        let mut score_board = heuristic::evaluate_board(&mut board);
        println!("// Initial configuration:");
        print_board(&mut board);
        print_score_board(&mut score_board);
        let (x, y) = to_remove;
        remove_last_pawn(
            &mut board,
            &mut score_board,
            x,
            y,
            actual_player,
            removed,
            &table,
            &mut zhash,
        );
        println!("// final configuration:");
        print_board(&mut board);
        print_score_board(&mut score_board);
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

    fn test_change_board_hint(
        white_pos: Vec<(usize, usize)>,
        black_pos: Vec<(usize, usize)>,
        to_remove: (usize, usize),
        actual_player: Option<bool>,
        removed: Vec<((isize, isize), (isize, isize))>,
    ) -> bool {
        let mut board = [[None; SIZE_BOARD]; SIZE_BOARD];
        white_pos
            .iter()
            .for_each(|&(x, y)| board[x][y] = Some(true));
        black_pos
            .iter()
            .for_each(|&(x, y)| board[x][y] = Some(false));
        let mut score_board = heuristic::evaluate_board(&mut board);
        println!("// Initial configuration:");
        print_board(&mut board);
        print_score_board(&mut score_board);
        let (x, y) = to_remove;
        let res = change_board_hint(&mut board, &mut score_board, x, y, actual_player);
        println!("// final configuration:");
        print_board(&mut board);
        print_score_board(&mut score_board);
        if removed.len() != res.len() {
            return false;
        }
        for i in 0..removed.len() {
            let ((x1, y1), (x2, y2)) = removed[i];
            let ((a1, b1), (a2, b2)) = removed[i];
            if x1 != a1 || x2 != a2 || y1 != b1 || y2 != b2 {
                return false;
            }
        }
        true
    }

    #[test]
    fn test_change_board_hint_00() {
        let black_pos = vec![(2, 0), (2, 1), (2, 2), (2, 3), (2, 4), (5, 5)];
        let white_pos = vec![(0, 2), (1, 2), (3, 2), (4, 2), (3, 3), (4, 4)];
        let to_remove = (2, 2);
        let actual_player = Some(false);
        let removed = vec![((0, 0), (1, 1))];
        assert!(!test_change_board_hint(
            black_pos,
            white_pos,
            to_remove,
            actual_player,
            removed
        ));
    }
    //pub fn change_board_hint(
    //    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    //    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    //    x: usize,
    //    y: usize,
    //    pawn: Option<bool>,
    //) -> Vec<((isize, isize), (isize, isize))> {
}
