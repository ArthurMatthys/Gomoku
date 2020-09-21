use super::super::checks::after_turn_check::DIRECTIONS;
use super::super::checks::capture::DIRS;
use super::super::checks::double_three::check_double_three_hint;
use super::super::render::board::SIZE_BOARD;

use super::heuristic;
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

macro_rules! get_bool {
    ($e:expr) => {
        match $e {
            Some(true) => "T",
            Some(false) => "F",
            None => "N",
        }
    };
}
macro_rules! index_edge {
    ($delta:expr) => {
        match delta {
            (1, 1) => 1,
            (1, 0) => 1,
            (1, -1) => 1,
            (0, 1) => 1,
            (0, -1) => 0,
            (-1, 1) => 0,
            (-1, 0) => 0,
            (-1, -1) => 0,
        }
    };
}

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
            break;
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
    len_align: u8,
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

fn change_score_board_remove(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    x: isize,
    y: isize,
) {
    let pawn = board[x as usize][y as usize];
    for (i, (dx, dy)) in DIRECTIONS.iter().enumerate() {
        score_board[x as usize][y as usize][i] = (0, Some(false), Some(false));
        for way in [-1, 1].iter() {
            let new_x = x + (way * dx);
            let new_y = y + (way * dy);
            if valid_coord!(new_x, new_y) {
                match board[new_x as usize][new_y as usize] {
                    //                   None => (println!("Hello3")),
                    a if a == pawn => {
                        let (align, mut left_edge, mut right_edge) =
                            score_board[new_x as usize][new_y as usize][i];
                        if *way == -1 {
                            right_edge = Some(false);
                        } else {
                            left_edge = Some(false);
                        }
                        //                        println!("hello");
                        decrease_align(
                            board,
                            score_board,
                            pawn,
                            (x, y),
                            (&(way * dx), &(way * dy)),
                            i,
                            align,
                            left_edge,
                            right_edge,
                        );
                    }
                    a if a != pawn => {
                        let (align, mut left_edge, mut right_edge) =
                            score_board[new_x as usize][new_y as usize][i];
                        if *way == -1 {
                            right_edge = Some(false);
                        } else {
                            left_edge = Some(false);
                        }
                        //                        println!("hello2");
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

pub fn change_score_board_add(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    x: isize,
    y: isize,
) {
    let pawn = board[x as usize][y as usize];
    let get_edge_case = |actual: &Option<bool>, x: &isize, y: &isize| {
        if valid_coord!(*x, *y) {
            match board[*x as usize][*y as usize] {
                None => 1,
                a if a == *actual => 2,
                a if a != *actual => 3,
                _ => unreachable!(),
            }
        } else {
            0
        }
    };
    for (i, (dx, dy)) in DIRECTIONS.iter().enumerate() {
        let mut tot_align = 1;
        let x_left = x - dx;
        let y_left = y - dy;
        let edge_case_left = get_edge_case(&pawn, &x_left, &y_left);
        let (mut align_left, mut left_edge_left): (u8, Option<bool>) = (0, None);
        match edge_case_left {
            0 => (),
            1 => left_edge_left = Some(false),
            2 => {
                let focused_tuple = score_board[x_left as usize][y_left as usize][i];
                align_left = focused_tuple.0;
                left_edge_left = focused_tuple.1;
                tot_align += align_left;
            }
            3 => {
                align_left = score_board[x_left as usize][y_left as usize][i].0;
                left_edge_left = Some(true);
            }
            _ => unreachable!(),
        };
        let x_right = x + dx;
        let y_right = y + dy;
        let edge_case_right = get_edge_case(&pawn, &x_right, &y_right);
        let (mut align_right, mut right_edge_right): (u8, Option<bool>) = (0, None);
        match edge_case_right {
            0 => (),
            1 => {
                right_edge_right = Some(false);
            }
            2 => {
                let focused_tuple = score_board[x_right as usize][y_right as usize][i];
                align_right = focused_tuple.0;
                right_edge_right = focused_tuple.2;
                tot_align += align_right;
            }
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
        score_board[x as usize][y as usize][i] = (tot_align, left_edge_left, right_edge_right);
        match edge_case_left {
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
        match edge_case_right {
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
                for &(dx, dy) in DIRS.iter() {
                    let new_x = x as isize + dx;
                    let new_y = y as isize + dy;
                    let mut check = 0;
                    let mut score = 0i64;
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
                                } else if board[opp_x as usize][opp_y as usize] == None {
                                    edge_opp = Some(false);
                                } else {
                                    edge_opp = Some(true);
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
) -> Option<(usize, usize)> {
    if *depth < 0 || *depth < *depth_win {
        return None;
    }
    let threats: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
        threat_search_space(board, score_board, player_actual, player_actual_catch);

    if threats.len() == 0 {
        return None;
    }

    if threats.len() == 1 && threats[0].1 == TypeOfThreat::WIN {
        return Some(threats[0].0);
    }

    for (threat, typeofthreat, counters) in threats.iter() {
        let (x, y) = threat;
        let removed = change_board_hint(board, score_board, *x, *y, player_actual);
        *player_actual_catch += removed.len() as isize;

        //        println!(
        //            "Config depth {}, black pawn in ({},{}), threat : {} ",
        //            depth,
        //            x,
        //            y,
        //            match typeofthreat {
        //                TypeOfThreat::FIVE_TAKE => "FIVE_TAKE",
        //                TypeOfThreat::FOUR_TAKE => "FOUR_TAKE",
        //                TypeOfThreat::THREE_TAKE => "THREE_TAKE",
        //                TypeOfThreat::TWO_TAKE => "TWO_TAKE",
        //                TypeOfThreat::ONE_TAKE => "ONE_TAKE",
        //                TypeOfThreat::FOUR_O => "FOUR_O",
        //                TypeOfThreat::FOUR_SO => "FOUR_SO",
        //                TypeOfThreat::THREE_O => "THREE_O",
        //                TypeOfThreat::WIN => "WIN",
        //                TypeOfThreat::EMPTY => "EMPTY",
        //            }
        //        );
        //        for i in 0..19 {
        //            for j in 0..19 {
        //                if i == *y && j == *x {
        //                    print!("⊛")
        //                } else {
        //                    match board[j][i] {
        //                        Some(true) => print!("⊖"),
        //                        Some(false) => print!("⊕"),
        //                        None => print!("_"),
        //                    }
        //                }
        //            }
        //            println!();
        //        }
        let mut counters_valid: Vec<(usize, usize)> = vec![];
        for (opp_x, opp_y) in counters.iter() {
            if !check_double_three_hint(board, player_actual, *opp_x as isize, *opp_y as isize) {
                counters_valid.push((*opp_x, *opp_y));
            }
        }
        //        print!("Counters : ");
        //        counters_valid
        //            .iter()
        //            .for_each(|(c_x, c_y)| print!("({},{}); ", c_x, c_y));
        //        println!();

        if counters_valid.len() == 0 && *typeofthreat < TypeOfThreat::FIVE_TAKE {
            for (x, y) in find_available_pos(board, get_opp!(player_actual)) {
                if !counters_valid
                    .iter()
                    .any(|&(cmp_x, cmp_y)| cmp_x == x && cmp_y == y)
                {
                    counters_valid.push((x, y));
                }
            }
        }

        //        print!("Counters : ");
        //        counters_valid
        //            .iter()
        //            .for_each(|(c_x, c_y)| print!("({},{}); ", c_x, c_y));
        //        println!();

        let mut win: bool = true;

        for (counter_x, counter_y) in counters_valid.iter() {
            let removed_counter = change_board_hint(
                board,
                score_board,
                *counter_x,
                *counter_y,
                get_opp!(player_actual),
            );
            //            println!(
            //                "Config depth {}, white pawn in ({},{}) ",
            //                depth, counter_x, counter_y
            //            );
            //            for i in 0..19 {
            //                for j in 0..19 {
            //                    if i == *counter_y && j == *counter_x {
            //                        print!("⊙")
            //                    } else {
            //                        match board[j][i] {
            //                            Some(true) => print!("⊖"),
            //                            Some(false) => print!("⊕"),
            //                            None => print!("_"),
            //                        }
            //                    }
            //                }
            //                println!();
            //            }

            *player_opposite_catch += removed_counter.len() as isize;
            let res = find_continuous_threats(
                board,
                score_board,
                player_actual,
                player_actual_catch,
                player_opposite_catch,
                &mut (*depth - 2),
                depth_win,
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
            //            println!();
            //            println!();

            if res == None {
                win = false;
            }
        }
        *player_actual_catch -= removed.len() as isize;
        remove_last_pawn_hint(board, score_board, *x, *y, player_actual, removed);

        if win {
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

pub fn board_state_win(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
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
    board[x][y] = None;
    removed.iter().for_each(|&((x1, y1), (x2, y2))| {
        board[x1 as usize][y1 as usize] = old;
        change_score_board_add(board, score_board, x1 as isize, y1 as isize);
        board[x2 as usize][y2 as usize] = old;
        change_score_board_add(board, score_board, x2 as isize, y2 as isize);
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
    board[x][y] = pawn;
    change_score_board_add(board, score_board, x as isize, y as isize);
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
            board[x1 as usize][y1 as usize] = None;
            change_score_board_remove(board, score_board, x2 as isize, y2 as isize);
            board[x2 as usize][y2 as usize] = None;
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
    board[x][y] = None;
    add_zhash!(table, zhash, x, y, get_zindex_from_pawn!(pawn));
    removed.iter().for_each(|&((x1, y1), (x2, y2))| {
        add_zhash!(table, zhash, x1, y1, get_zindex_from_pawn!(old));
        board[x1 as usize][y1 as usize] = old;
        change_score_board_add(board, score_board, x1 as isize, y1 as isize);
        add_zhash!(table, zhash, x2, y2, get_zindex_from_pawn!(old));
        board[x2 as usize][y2 as usize] = old;
        change_score_board_add(board, score_board, x2 as isize, y2 as isize);
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
    board[x][y] = pawn;
    change_score_board_add(board, score_board, x as isize, y as isize);
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
            board[x1 as usize][y1 as usize] = None;
            add_zhash!(table, zhash, x1, y1, get_zindex_from_pawn!(opp));
            change_score_board_remove(board, score_board, x2 as isize, y2 as isize);
            board[x2 as usize][y2 as usize] = None;
            add_zhash!(table, zhash, x2, y2, get_zindex_from_pawn!(opp));
            removed.push(((x1, y1), (x2, y2)));
        }
    }

    removed
}
#[cfg(test)]
mod tests {
    use super::*;

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
        let mut score_board = heuristic::evaluate_board(&mut test_board);
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
        board_state_win(&mut test_board, &mut score_tab, actual_take, opp_take)
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

    fn test_score_board(
        history_pos: Vec<(usize, usize)>,
        history_remove: Vec<(usize, usize)>,
    ) -> bool {
        let mut test_board = [[None; SIZE_BOARD]; SIZE_BOARD];
        let mut score_board: [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD] =
            [[[(0, Some(false), Some(false)); 4]; SIZE_BOARD]; SIZE_BOARD];
        let mut pawn = Some(false);

        for &(x, y) in history_pos.iter() {
            test_board[x][y] = pawn;
            change_score_board_add(&mut test_board, &mut score_board, x as isize, y as isize);
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
                        Some(true) => print!("W"),
                        Some(false) => print!("B"),
                        None => print!("E"),
                    }
                    score_board[j][i].iter().for_each(|&(value, a, b)| {
                        print!("{:2}{}{}", value, get_bool!(a), get_bool!(b))
                    });
                    print!(" ");
                }
                println!();
            }
            if pawn == Some(false) {
                pawn = Some(true);
            } else {
                pawn = Some(false);
            }
            println!("-----------");
        }
        for &(x, y) in history_remove.iter() {
            change_score_board_remove(&mut test_board, &mut score_board, x as isize, y as isize);
            test_board[x][y] = None;
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
                        Some(true) => print!("W"),
                        Some(false) => print!("B"),
                        None => print!("E"),
                    }
                    score_board[j][i].iter().for_each(|&(value, a, b)| {
                        print!("{:2}{}{}", value, get_bool!(a), get_bool!(b))
                    });
                    print!(" ");
                }
                println!();
            }
            if pawn == Some(false) {
                pawn = Some(true);
            } else {
                pawn = Some(false);
            }
            println!("-----------");
        }
        false
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
        for i in 0..19 {
            print!("// ");
            for j in 0..19 {
                for dir in 0..4 {
                    print!("{}", score_board[j][i][dir].0)
                }
                print!("  ")
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
        let expected_result = Some((0, 0));

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
}
