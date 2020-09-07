use super::super::checks::after_turn_check::DIRECTIONS;
use super::super::checks::capture::DIRS;
use super::super::checks::double_three::check_double_three_hint;
use super::super::render::board::SIZE_BOARD;
use super::heuristic;
// use super::handle_board::*;

macro_rules! valid_coord {
    (
        $x: expr,
        $y: expr
    ) => {
        $x >= 0 && $x < SIZE_BOARD as isize && $y >= 0 && $y < SIZE_BOARD as isize
    };
}

macro_rules! explore_align {
    (
        $board: expr,
        $record: expr,
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
            $record[$new_line as usize][$new_col as usize][$dir] = false;
        }
    };
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

macro_rules! explore_one {
    (
        $new_line: expr,
        $new_col: expr,
        $dir: expr,
        $orientation: expr
    ) => {
        (
            $new_line + (DIRECTIONS[$dir].0 * $orientation),
            $new_col + (DIRECTIONS[$dir].1 * $orientation),
        )
    };
}

macro_rules! flatten {
    (
        $vec: expr
    ) => {
        $vec.into_iter().flatten().collect()
    };
}

const AVRG_MAX_MULTIPLE_THREATS: usize = 2;
const MAX_MULTIPLE_DEFENSE_MOVES: usize = 4;

#[derive(Copy, Clone, PartialEq)]
enum TypeOfThreat {
    // NONE,
    THREE_O,
    FOUR_O,
    FOUR_SO,
    FIVE_TAKE,
    FIVE,
    TAKE,
}

// Aim of function :
// Initialize a record for efficient tracking of modifications afterwards
// Sets adversatory and empty positions to false and the player's ones to true
// (for each direction at a given position)
fn initialize_record(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    actual_player: Option<bool>,
) -> [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD] {
    let mut record = [[[false; 4]; SIZE_BOARD]; SIZE_BOARD];

    for line in 0..SIZE_BOARD {
        for col in 0..SIZE_BOARD {
            if board[line][col] == actual_player {
                for dir in 0..4 {
                    record[line][col][dir] = true;
                }
            }
        }
    }
    record
}

fn capture_blank(
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    actual_player: Option<bool>,
    x: usize,
    y: usize,
    old_dir: usize,
) -> Vec<(usize, usize)> {
    let mut ret = vec![];

    for dir in 0..4 {
        if old_dir == dir {
            continue;
        }
        let (dx, dy) = DIRECTIONS[dir];
        let mut complete = 0;
        let mut empty = 0;
        for way in [-1, 1].iter() {
            let new_x = x as isize + way * dx;
            let new_y = y as isize + way * dy;
            if !valid_coord!(new_x, new_y) {
                break;
            }
            match board[new_x as usize][new_y as usize] {
                None => {
                    complete += 1;
                    empty = *way;
                }
                a if a == actual_player => {
                    let (align, edge_l, edge_r) = score_board[new_x as usize][new_y as usize][dir];
                    if align == 1 {
                        match *way {
                            -1 => {
                                if edge_l == Some(true) {
                                    complete += 2;
                                }
                            }
                            1 => {
                                if edge_r == Some(true) {
                                    complete += 2;
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                _ => (),
            }
        }
        if complete == 3 {
            match empty {
                -1 => ret.push((x - dx as usize, y - dy as usize)),
                1 => ret.push((x + dx as usize, y + dy as usize)),
                _ => unreachable!(),
            }
        }
    }
    ret
}

fn capture_coordinates(
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    actual_player: Option<bool>,
    x: usize,
    y: usize,
    dir: usize,
) -> Vec<(usize, usize)> {
    let mut coordinates: Vec<(usize, usize)> = Vec::with_capacity(4);

    for new_dir in 0..4 {
        if dir == new_dir {
            continue;
        } else {
            let (mut new_line, mut new_col): (isize, isize) = (x as isize, y as isize);

            match score_board[x][y][new_dir] {
                (2, Some(true), Some(false)) => {
                    explore_align_light!(board, new_line, new_col, actual_player, new_dir, 1);
                    coordinates.push((new_line as usize, new_col as usize));
                }
                (2, Some(false), Some(true)) => {
                    explore_align_light!(board, new_line, new_col, actual_player, new_dir, -1);
                    coordinates.push((new_line as usize, new_col as usize));
                }
                _ => continue,
            }
        }
    }
    coordinates
}

fn capture_coordinates_vec(
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    actual_player: Option<bool>,
    coord: Vec<(usize, usize)>,
    dir: usize,
) -> Vec<(usize, usize)> {
    flatten!(coord
        .iter()
        .map(|&(x, y)| capture_coordinates(score_board, board, actual_player, x, y, dir))
        .collect::<Vec<Vec<(usize, usize)>>>())
}

fn explore_and_find_threats(
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    limit: usize,
    orientation: isize,
    cline: &isize,
    ccol: &isize,
    threat: TypeOfThreat,
    actual_player: Option<bool>,
    dir: usize,
    all_threats: &mut Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)>,
    new_line: &isize,
    new_col: &isize,
) -> () {
    let mut tmp_positions: Vec<Vec<(usize, usize)>> = vec![];
    for expansion in 0..limit {
        tmp_positions.push(capture_coordinates(
            score_board,
            board,
            actual_player,
            *cline as usize,
            *ccol as usize,
            dir,
        ));
        explore_one!(cline, ccol, dir, orientation);
    }
    all_threats.push((
        (*new_line as usize, *new_col as usize),
        threat,
        flatten!(tmp_positions),
    ));
}

// (line, col): (usize, usize),
// score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
// board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
// record: &mut [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD],
// actual_player: Option<bool>,
// actual_take: &mut isize,
// dir: usize
fn manage_so(
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    record: &mut [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD],
    actual_player: Option<bool>,
    dir: usize,
    mut new_line: isize,
    mut new_col: isize,
    way: isize,
    opp_way: isize,
    threat: TypeOfThreat,
    all_threats: &mut Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)>,
) -> () {
    explore_align!(board, record, new_line, new_col, actual_player, dir, way);
    let (nline, ncol) = explore_one!(new_line, new_col, dir, way);
    let (mut cline, mut ccol) = (new_line, new_col);
    // retrieve defensive moves
    if valid_coord!(nline, ncol)
        && board[nline as usize][ncol as usize] == actual_player
        && score_board[nline as usize][ncol as usize][dir].0 > 0
    {
        match score_board[nline as usize][ncol as usize][dir].0 {
            x if x >= 5 => {} // WIN FOR SURE!!!!!!!!
            4 => {
                all_threats.push((
                    (new_line as usize, new_col as usize),
                    threat,
                    capture_coordinates(
                        score_board,
                        board,
                        actual_player,
                        new_line as usize,
                        new_col as usize,
                        dir,
                    ),
                ));
            }
            a if a == 3 || a == 2 || a == 1 => explore_and_find_threats(
                score_board,
                board,
                (5 - a) as usize,
                opp_way,
                &cline,
                &ccol,
                threat,
                actual_player,
                dir,
                all_threats,
                &new_line,
                &new_col,
            ),
            _ => unreachable!(),
        }
    } else {
        explore_and_find_threats(
            score_board,
            board,
            5,
            opp_way,
            &cline,
            &ccol,
            threat,
            actual_player,
            dir,
            all_threats,
            &new_line,
            &new_col,
        );
    }
}

fn connect_4(
    (line, col): (usize, usize),
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    record: &mut [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD],
    actual_player: Option<bool>,
    actual_take: &mut isize,
    dir: usize,
) -> Option<Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)>> {
    let mut new_line: isize = line as isize;
    let mut new_col: isize = col as isize;
    let mut all_threats: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![];

    if record[line][col][dir] {
        match (score_board[line][col][dir].1, score_board[line][col][dir].2) {
            (Some(true), Some(false)) | (None, Some(false)) => {
                manage_so(
                    score_board,
                    board,
                    record,
                    actual_player,
                    dir,
                    new_line,
                    new_col,
                    -1,
                    1,
                    TypeOfThreat::FOUR_SO,
                    &mut all_threats,
                );
                Some(all_threats)
            }
            (Some(false), Some(true)) | (Some(false), None) => {
                manage_so(
                    score_board,
                    board,
                    record,
                    actual_player,
                    dir,
                    new_line,
                    new_col,
                    1,
                    -1,
                    TypeOfThreat::FOUR_SO,
                    &mut all_threats,
                );
                Some(all_threats)
            }
            (Some(false), Some(false)) => {
                let mut new_line2: isize = line as isize;
                let mut new_col2: isize = col as isize;
                manage_so(
                    score_board,
                    board,
                    record,
                    actual_player,
                    dir,
                    new_line,
                    new_col,
                    -1,
                    1,
                    TypeOfThreat::FOUR_O,
                    &mut all_threats,
                );
                manage_so(
                    score_board,
                    board,
                    record,
                    actual_player,
                    dir,
                    new_line2,
                    new_col2,
                    -1,
                    1,
                    TypeOfThreat::FOUR_O,
                    &mut all_threats,
                );
                Some(all_threats)
            }
            _ => None,
        }
    } else {
        None
    }
}

// fn connect_3(
//     (line, col): (usize, usize),
//     board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
//     record: &mut [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD],
//     actual_player: Option<bool>,
//     actual_take: &mut isize,
//     left_side: Option<bool>,
//     right_side: Option<bool>,
//     dir: usize
// ) -> () {
// // ) -> Vec<((usize,usize), TypeOfThreat, Vec<(usize,usize)>)> {
//     let all_threats:Vec<((usize,usize), TypeOfThreat, Vec<(usize,usize)>)> = vec![];

//     if record[line][col][dir] {
//         for orientation in [-1, 1].iter() {
//             let mut new_line: isize = line as isize;
//             let mut new_col: isize = col as isize;
//             while valid_coord!(new_line, new_col) && board[new_line as usize][new_col as usize] == actual_player {
//                 new_line = new_line + (DIRECTIONS[dir].0 * orientation);
//                 new_col = new_col + (DIRECTIONS[dir].1 * orientation);
//                 record[new_line as usize][new_col as usize][dir] = false;
//             }
//             if valid_coord!(new_line, new_col) && board[new_line as usize][new_col as usize] == None {
//                 let del_line = new_line + (DIRECTIONS[dir].0 * orientation);
//                 let del_col = new_col + (DIRECTIONS[dir].1 * orientation);
//                 if valid_coord!(del_line, del_col) && board[del_line as usize][del_col as usize] == None {

//                 }
//             }
//         }
//     }
// }
// (
//     Vec<(usize, usize)>,
//     Vec<((usize, usize), Vec<(usize, usize)>)>,
// )
//

macro_rules! get_edges {
    (
        $way: expr,
        $tuple: expr
    ) => {
        match $way {
            -1 => ($tuple.1, $tuple.2),
            1 => ($tuple.2, $tuple.1),
            _ => unreachable!(),
        }
    };
}

fn create_align(
    steps: Vec<isize>,
    way: isize,
    (x, y): (isize, isize),
    (dx, dy): (isize, isize),
) -> Vec<(usize, usize)> {
    let mut ret: Vec<(usize, usize)> = vec![];
    //let mut ret: Vec<(isize, isize)> = Vec::with_capacity($steps.len());
    steps.iter().for_each(|&step| {
        ret.push((
            (x + way * dx * step) as usize,
            (y + way * dy * step) as usize,
        ))
    });
    return ret;
}

fn connect_2(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    record: &mut [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD],
    actual_player: Option<bool>,
    (x, y): (usize, usize),
    dir: usize,
) -> Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> {
    let mut ret = vec![];
    let focused_tuple = score_board[x][y][dir as usize];
    let (dx, dy) = DIRECTIONS[dir];
    for way in [-1, 1].iter() {
        let (actual_edge, opp_edge): (Option<bool>, Option<bool>) = get_edges!(way, focused_tuple);
        if actual_edge != Some(false) {
            continue;
        }
        let mut space = 1;
        let mut align_ally = 0;
        let mut new_x = x as isize;
        let mut new_y = y as isize;
        explore_align!(board, record, new_x, new_y, actual_player, dir, way);
        // explore_align_light!(board, new_x, new_y, actual_player, dir, way);
        let mut cursor_x = new_x;
        let mut cursor_y = new_y;
        loop {
            cursor_x += dx * way;
            cursor_y += dy * way;
            if !valid_coord!(cursor_x, cursor_y) || space >= 3 || align_ally != 0 {
                break;
            }
            match board[cursor_x as usize][cursor_y as usize] {
                None => space += 1,
                a if a == actual_player => {
                    align_ally = score_board[cursor_x as usize][cursor_y as usize][dir].0
                }
                _ => break,
            }
        }
        let mut gather_infos: Vec<(
            (usize, usize),
            TypeOfThreat,
            Vec<(usize, usize)>,
            Vec<(usize, usize)>,
        )> = Vec::with_capacity(2);

        match opp_edge {
            Some(false) => match space {
                1 => match align_ally {
                    0 => (),
                    1 => {
                        // _00_0?
                        //opened edge, 1 space, 1 more pawn
                        let (ally_edge, _) = get_edges!(
                            way,
                            score_board[(new_x + way * dx) as usize][(new_y + way * dy) as usize]
                                [dir]
                        );
                        let steps: Vec<isize> = vec![-2, -1, 1];
                        let (opp_steps, threat) = match ally_edge {
                            Some(false) => (vec![], TypeOfThreat::FOUR_O),
                            _ => (vec![-3, 1], TypeOfThreat::FOUR_SO),
                        };
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            threat,
                            create_align(steps, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    2 => {
                        // _00_00?
                        //opened edge, 1 space, 2 more pawn
                        let steps = vec![-2, -1, 1, 2];
                        let (opp_steps, threat) = (vec![], TypeOfThreat::FIVE_TAKE);
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            threat,
                            create_align(steps, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    3 => {
                        // _00_000?
                        //opened edge, 1 space, 3 more pawn
                        let steps = vec![-1, 1, 2];
                        let (opp_steps, threat) = (vec![], TypeOfThreat::FIVE_TAKE);
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            threat,
                            create_align(steps, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    4 => {
                        // _00_0000?
                        //opened edge, 1 space, 4 more pawn
                        let steps = vec![1, 2];
                        let (opp_steps, threat) = (vec![], TypeOfThreat::FIVE_TAKE);
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            threat,
                            create_align(steps, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    5 => (),
                    _ => unreachable!(),
                },
                2 => match align_ally {
                    0 => {
                        // _00__#
                        //opened edge, 2 space, 0 more pawn
                        let (mut tmp_x, mut tmp_y) = (x as isize, y as isize);
                        explore_align_light!(board, tmp_x, tmp_y, actual_player, dir, -way);
                        tmp_x -= way * dx;
                        tmp_y -= way * dy;
                        if !valid_coord!(tmp_x, tmp_y)
                            || board[tmp_x as usize][tmp_y as usize] != actual_player
                        {
                            ();
                        } else {
                            let steps_no_space = vec![-2, -1];
                            let opp_steps_no_space = vec![-3, 1];
                            gather_infos.push((
                                (new_x as usize, new_y as usize),
                                TypeOfThreat::THREE_O,
                                create_align(steps_no_space, *way, (new_x, new_y), (dx, dy)),
                                create_align(opp_steps_no_space, *way, (new_x, new_y), (dx, dy)),
                            ));
                            let steps_with_space = vec![-2, -1];
                            let opp_steps_with_space = vec![0];
                            gather_infos.push((
                                ((new_x + way * dx) as usize, (new_y + way * dy) as usize),
                                TypeOfThreat::THREE_O,
                                create_align(steps_with_space, *way, (new_x, new_y), (dx, dy)),
                                create_align(opp_steps_with_space, *way, (new_x, new_y), (dx, dy)),
                            ));
                        }
                    }
                    1 => {
                        // _00__0?
                        //opened edge, 2 space, 1 more pawn
                        let steps_no_space = vec![-2, -1, 2];
                        let opp_steps_no_space = vec![-3, 1];
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            TypeOfThreat::FOUR_SO,
                            create_align(steps_no_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_no_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                        let steps_with_space = vec![-2, -1, 2];
                        let opp_steps_with_space = vec![0];
                        gather_infos.push((
                            ((new_x + way * dx) as usize, (new_y + way * dy) as usize),
                            TypeOfThreat::FOUR_SO,
                            create_align(steps_with_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_with_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    2 => {
                        // _00__00?
                        //opened edge, 2 space, 2 more pawn
                        let steps_no_space = vec![-1, 2];
                        let opp_steps_no_space = vec![-3, 1];
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            TypeOfThreat::FIVE_TAKE,
                            create_align(steps_no_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_no_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                        let steps_with_space = vec![-1, 2];
                        let opp_steps_with_space = vec![0];
                        gather_infos.push((
                            ((new_x + way * dx) as usize, (new_y + way * dy) as usize),
                            TypeOfThreat::FIVE_TAKE,
                            create_align(steps_with_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_with_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    3..=4 => {
                        // _00__000?
                        // _00__0000?
                        //opened edge, 2 space, 3 or 4 more pawn
                        let steps_no_space = vec![2];
                        let opp_steps_no_space = vec![-3, 1];
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            TypeOfThreat::FIVE_TAKE,
                            create_align(steps_no_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_no_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                        let steps_with_space = vec![2];
                        let opp_steps_with_space = vec![0];
                        gather_infos.push((
                            ((new_x + way * dx) as usize, (new_y + way * dy) as usize),
                            TypeOfThreat::FIVE_TAKE,
                            create_align(steps_with_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_with_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    5 => (),
                    _ => unreachable!(),
                },
                3 => {
                    // _00___?
                    //opened edge, 3 space, 2 in a row alone
                    let steps_no_space = vec![-2, -1];
                    let opp_steps_no_space = vec![-3, 1];
                    gather_infos.push((
                        (new_x as usize, new_y as usize),
                        TypeOfThreat::THREE_O,
                        create_align(steps_no_space, *way, (new_x, new_y), (dx, dy)),
                        create_align(opp_steps_no_space, *way, (new_x, new_y), (dx, dy)),
                    ));
                    let steps_with_space = vec![-2, -1];
                    let opp_steps_with_space = vec![0];
                    gather_infos.push((
                        ((new_x + way * dx) as usize, (new_y + way * dy) as usize),
                        TypeOfThreat::THREE_O,
                        create_align(steps_with_space, *way, (new_x, new_y), (dx, dy)),
                        create_align(opp_steps_with_space, *way, (new_x, new_y), (dx, dy)),
                    ));
                }
                _ => unreachable!(),
            },
            _ => match space {
                1 => match align_ally {
                    1 => {
                        // #00_0?
                        let (ally_edge, _) = get_edges!(
                            way,
                            score_board[(new_x + way * dx) as usize][(new_y + way * dy) as usize]
                                [dir]
                        );
                        if ally_edge == Some(false) {
                            let steps: Vec<isize> = vec![-2, -1, 1];
                            let opp_steps = vec![-3, 1];
                            gather_infos.push((
                                (new_x as usize, new_y as usize),
                                TypeOfThreat::FOUR_SO,
                                create_align(steps, *way, (new_x, new_y), (dx, dy)),
                                create_align(opp_steps, *way, (new_x, new_y), (dx, dy)),
                            ));
                        } else {
                            continue;
                        }
                    }
                    2 => {
                        // #00_00?
                        let steps = vec![-2, -1, 1, 2];
                        let (opp_steps, threat) = (vec![], TypeOfThreat::FIVE_TAKE);
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            threat,
                            create_align(steps, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    3 => {
                        // #00_000?
                        let steps = vec![-1, 1, 2];
                        let (opp_steps, threat) = (vec![], TypeOfThreat::FIVE_TAKE);
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            threat,
                            create_align(steps, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    4 => {
                        // #00_0000?
                        let steps = vec![1, 1];
                        let (opp_steps, threat) = (vec![], TypeOfThreat::FIVE_TAKE);
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            threat,
                            create_align(steps, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    0 => (),
                    5 => (),
                    _ => unreachable!(),
                },
                2 => match align_ally {
                    0 => {
                        // #00__#
                        continue;
                    }
                    1 => {
                        // #00__0?
                        let steps_no_space = vec![-2, -1, 2];
                        let opp_steps_no_space = vec![1];
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            TypeOfThreat::FOUR_SO,
                            create_align(steps_no_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_no_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                        let steps_with_space = vec![-2, -1, 2];
                        let opp_steps_with_space = vec![0];
                        gather_infos.push((
                            ((new_x + way * dx) as usize, (new_y + way * dy) as usize),
                            TypeOfThreat::FOUR_SO,
                            create_align(steps_with_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_with_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    2 => {
                        // #00__00?
                        let steps_no_space = vec![-1, 2];
                        let opp_steps_no_space = vec![1];
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            TypeOfThreat::FIVE_TAKE,
                            create_align(steps_no_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_no_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                        let steps_with_space = vec![-1, 2];
                        let opp_steps_with_space = vec![0];
                        gather_infos.push((
                            ((new_x + way * dx) as usize, (new_y + way * dy) as usize),
                            TypeOfThreat::FIVE_TAKE,
                            create_align(steps_with_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_with_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    3..=4 => {
                        // #00__000?
                        // #00__0000?
                        let steps_no_space = vec![2];
                        let opp_steps_no_space = vec![1];
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            TypeOfThreat::FIVE_TAKE,
                            create_align(steps_no_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_no_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                        let steps_with_space = vec![2];
                        let opp_steps_with_space = vec![0];
                        gather_infos.push((
                            ((new_x + way * dx) as usize, (new_y + way * dy) as usize),
                            TypeOfThreat::FIVE_TAKE,
                            create_align(steps_with_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_with_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    5 => (),
                    _ => unreachable!(),
                },
                3 => {
                    // #00___?
                    continue;
                }
                _ => unreachable!(),
            },
        };
    }
    //        ---_----
    //        if align_ally > 0 && space == 1 {
    //            let mut capture_extra_no_space: Vec<(usize, usize)> = vec![];
    //            if align_ally != 4 {
    //                capture_extra_no_space = capture_blank(
    //                    score_board,
    //                    board,
    //                    actual_player,
    //                    new_x as usize,
    //                    new_y as usize,
    //                    dir,
    //                );
    //            }
    //            let mut capture_extra_with_space = capture_blank(
    //                score_board,
    //                board,
    //                actual_player,
    //                (new_x + *way * dx) as usize,
    //                (new_y + *way * dy) as usize,
    //                dir,
    //            );
    //            let mut opp_no_space = vec![];
    //            let mut opp_with_space = vec![];
    //            let mut push_coord_vec = |steps_no_space: Vec<isize>, steps_with_space: Vec<isize>| {
    //                steps_no_space.iter().for_each(|&step| {
    //                    opp_no_space.push((
    //                        (new_x + *way * dx * step) as usize,
    //                        (new_y + *way * dy * step) as usize,
    //                    ))
    //                });
    //                steps_with_space.iter().for_each(|&step| {
    //                    opp_with_space.push((
    //                        (new_x + *way * dx * step) as usize,
    //                        (new_y + *way * dy * step) as usize,
    //                    ))
    //                });
    //            };
    //            match align_ally {
    //                1 => {
    //                    let moves_no_space = vec![-2, -1, 2];
    //                    let moves_with_space = vec![-2, -1, 2];
    //                    push_coord_vec(moves_no_space, moves_with_space);
    //                    capture_extra_no_space
    //                        .push(((new_x + *way * dx) as usize, (new_y + *way * dy) as usize));
    //                    capture_extra_with_space.push((new_x as usize, new_y as usize));
    //                }
    //                2 => {
    //                    let moves_no_space = vec![-1, 2];
    //                    let moves_with_space = vec![-1, 2];
    //                    push_coord_vec(moves_no_space, moves_with_space);
    //                    capture_extra_no_space
    //                        .push(((new_x + *way * dx) as usize, (new_y + *way * dy) as usize));
    //                    capture_extra_with_space.push((new_x as usize, new_y as usize));
    //                }
    //                3 => {
    //                    let moves_no_space = vec![2];
    //                    let ally_tuple = score_board[(new_x + *way * dx * 2) as usize]
    //                        [(new_y * *way * dy * 2) as usize][dir];
    //                    let mut moves_with_space = vec![2];
    //                    if !(ally_tuple.1 == Some(false) && ally_tuple.2 == Some(false)) {
    //                        capture_extra_with_space.push((new_x as usize, new_y as usize));
    //                    }
    //                    capture_extra_no_space
    //                        .push(((new_x + *way * dx) as usize, (new_y + *way * dy) as usize));
    //                    push_coord_vec(moves_no_space, moves_with_space);
    //                }
    //                4 => {
    //                    let moves_no_space = vec![2];
    //                    let moves_with_space = vec![2, 3, 4, 5];
    //                    capture_extra_no_space
    //                        .push(((new_x + *way * dx) as usize, (new_y + *way * dy) as usize));
    //                    push_coord_vec(moves_no_space, moves_with_space);
    //                }
    //                _ => return vec![],
    //            }
    //            let capture_with_space =
    //                capture_coordinates_vec(score_board, board, actual_player, opp_with_space, dir);
    //            let capture_no_space =
    //                capture_coordinates_vec(score_board, board, actual_player, opp_no_space, dir);
    //            capture_with_space
    //                .iter()
    //                .for_each(|&x| capture_extra_with_space.push(x));
    //            capture_no_space
    //                .iter()
    //                .for_each(|&x| capture_extra_no_space.push(x));
    //        //TODO handle fusion
    //        //            let mut other_edge = None;
    //        //            if *way == -1 {
    //        //                other_edge = score_board[new_x as usize][new_y as usize][dir].1;
    //        //            } else {
    //        //                other_edge = score_board[new_x as usize][new_y as usize][dir].2;
    //        //            }
    //        //            explore_align_light!(board, new_x, new_y, actual_player, dir, way);
    //        } else if space >= 2 {
    //            //TODO handle formating 3o / 4 with empty in the middle
    //            let align_vec = vec![
    //                ((new_x - way * dx) as usize, (new_y - way * dy) as usize),
    //                (
    //                    (new_x - 2 * way * dx) as usize,
    //                    (new_y - 2 * way * dy) as usize,
    //                ),
    //            ];
    //            let capture_base =
    //                capture_coordinates_vec(score_board, board, actual_player, align_vec, dir);
    //            let mut capture_extra_no_space = capture_blank(
    //                score_board,
    //                board,
    //                actual_player,
    //                new_x as usize,
    //                new_y as usize,
    //                dir,
    //            );
    //            let mut capture_extra_with_space = capture_blank(
    //                score_board,
    //                board,
    //                actual_player,
    //                (new_x + *way * dx) as usize,
    //                (new_y + *way * dy) as usize,
    //                dir,
    //            );
    //            capture_base.iter().for_each(|&(x, y)| {
    //                capture_extra_no_space.push((x, y));
    //                capture_extra_with_space.push((x, y));
    //            });
    //
    //            capture_extra_with_space.push((new_x as usize, new_y as usize));
    //            capture_extra_no_space
    //                .push(((new_x + *way * dx) as usize, (new_y + *way * dy) as usize));
    //            capture_extra_no_space.push((
    //                (new_x - 3 * *way * dx) as usize,
    //                (new_y - 3 * *way * dy) as usize,
    //            ));
    //            ret.push((
    //                (new_x as usize, new_y as usize),
    //                TypeOfThreat::THREE_O,
    //                capture_extra_no_space,
    //            ));
    //            ret.push((
    //                ((new_x + *way * dx) as usize, (new_y + *way * dy) as usize),
    //                TypeOfThreat::THREE_O,
    //                capture_extra_with_space,
    //            ));
    //        }
    //TODO transfo gather_info into ret. gather_info : (pos, threat, alignement threat, possible
    //answer (without takes of alignement)
    ret
}

fn connect_3(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    (x, y): (usize, usize),
    dir: isize,
) -> Option<Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)>> {
    let mut ret = vec![];
    let focused_tuple = score_board[x][y][dir as usize];
    if focused_tuple.1 != Some(false) || focused_tuple.2 != Some(false) {
        return None;
    }

    Some(ret)
}

pub fn threat_search_space(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    actual_player: Option<bool>,
    actual_take: &mut isize,
) -> () {
    // 1. Initialize datastructures storing ready to be checked positions as well as threats
    let mut record: [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD] =
        initialize_record(board, score_board, actual_player);

    // 1.2. Initialize Threat board -> Vec containing with_capacity data (3 avrg max_possible threats per position) | (4 max defensive)
    // Optimized version of : [[Vec<(enum, Vec<(usize,usize)>)>; SIZE_BOARD]; SIZE_BOARD]
    let mut threat_board: Vec<Vec<Vec<(TypeOfThreat, Vec<(usize,usize)>)>>> = (0..SIZE_BOARD).map(|_|
                                                                                    (0..SIZE_BOARD).map(|_| 
                                                                                        Vec::with_capacity(AVRG_MAX_MULTIPLE_THREATS)
                                                                                    ).collect()
                                                                                ).collect();

    // 2. Parse board for actual_player's pawns
    for line in 0..SIZE_BOARD {
        for col in 0..SIZE_BOARD {
            if board[line][col] == actual_player {
                for dir in 0..4 {
                    // let ret: Vec<((usize,usize), TypeOfThreat, Vec<(usize,usize)>)> =
                    match score_board[line][col][dir].0 {
                        5 => (), //Instant win ?
                        4 => {
                            match connect_4(
                                (line, col),
                                score_board,
                                board,
                                &mut record,
                                actual_player,
                                actual_take,
                                dir,
                            ) {
                                None => (),
                                Some(x) => x.iter().for_each(|((x, y), typeOfThreat, Opp)| {
                                    threat_board[*x][*y].push((*typeOfThreat, vec![]));
                                    Opp.iter().for_each(|&opp| {
                                        let index = threat_board[*x][*y].len();
                                        threat_board[*x][*y][index].1.push(opp);
                                    });
                                }), // check borrow issue here !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                            }
                        }
                        3 => (),
                        2 => (),
                        _ => unreachable!(),
                    };
                    // ret.iter().for_each(|&((x,y), typeOfThreat, Opp)| threat_board[x][y].push((typeOfThreat, Opp)));
                }
                ()
            }
        }
    }

    // 5. Dispatch values inside the constructed datastructure in (1)

    // (
    //     Vec<(usize, usize)>,
    //     Vec<((usize, usize), Vec<(usize, usize)>)>,
    // )

    // [[Vec<(enum, Vec<(usize,usize)>)>; SIZE_BOARD]; SIZE_BOARD]
}

#[cfg(test)]
mod tests {
    use super::super::handle_board::change_score_board_add;
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


    fn test_capture_blank(
        white_pos: Vec<(usize, usize)>,
        black_pos: Vec<(usize, usize)>,
        actual_player: Option<bool>,
        x: usize,
        y: usize,
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
        let ret = capture_blank(&mut score_tab, &mut test_board, actual_player, x, y, 0);
        ret.len() > 0
    }

    #[test]
    fn blank_take0() {
        let x = 8;
        let y = 8;
        let black_pos = vec![(x + 1, y)];
        let white_pos = vec![(x + 2, y)];
        assert!(test_capture_blank(white_pos, black_pos, Some(false), x, y))
    }

    #[test]
    fn blank_take1() {
        let x = 8;
        let y = 8;
        let black_pos = vec![(x + 1, y)];
        let white_pos = vec![];
        assert!(!test_capture_blank(white_pos, black_pos, Some(false), x, y))
    }

    #[test]
    fn blank_take2() {
        let x = 8;
        let y = 8;
        let black_pos = vec![(x + 1, y), (x - 1, y)];
        let white_pos = vec![];
        assert!(!test_capture_blank(white_pos, black_pos, Some(false), x, y))
        // Compare output with given
    }

    #[test]
    fn blank_take3() {
        let x = 8;
        let y = 8;
        let black_pos = vec![(x + 1, y), (x - 1, y)];
        let white_pos = vec![(x + 2, y)];
        assert!(!test_capture_blank(white_pos, black_pos, Some(false), x, y))
    }

    #[test]
    fn blank_take4() {
        let x = 8;
        let y = 8;
        let black_pos = vec![];
        let white_pos = vec![];
        assert!(!test_capture_blank(white_pos, black_pos, Some(false), x, y))
    }

    fn test_threat(
        white_pos: Vec<(usize, usize)>,
        black_pos: Vec<(usize, usize)>,
        actual_take: &mut isize,
        opp_take: &mut isize,
        pos2check: (usize, usize),
        actual_player: Option<bool>,
        expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)>
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
        let mut score_board = heuristic::evaluate_board(&mut test_board);
        let mut record: [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD] = initialize_record(&mut test_board, &mut score_board, actual_player);
        // let mut threat_board: Vec<Vec<Vec<(TypeOfThreat, Vec<(usize,usize)>)>>> = (0..SIZE_BOARD).map(|_|
                                                                                //         (0..SIZE_BOARD).map(|_| 
                                                                                //             Vec::with_capacity(AVRG_MAX_MULTIPLE_THREATS)
                                                                                //         ).collect()
                                                                                //  ).collect();

        let mut tmp_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![];
        // for i in 0..19 {
        //     for j in 0..19 {
        //         match test_board[j][i] {
        //             Some(true) => print!("B"),
        //             Some(false) => print!("N"),
        //             None => print!("E"),
        //         }
        //         score_board[j][i].iter().for_each(|&(value, a, b)| {
        //             print!("{:2}{}{}", value, get_bool!(a), get_bool!(b))
        //         });
        //         print!(" ");
        //     }
        //     println!();
        // }
        for dir in 0..4 {
            tmp_result = match score_board[pos2check.0][pos2check.1][dir].0 {
                // 4 => { connect_4(pos2check, &mut score_board, &mut test_board, &mut record, actual_player, actual_take, dir) },
                // 3 => { connect_4(pos2check, &mut score_board, &mut test_board, &mut record, actual_player, actual_take, dir) },
                2 => { println!("OUPS_I_DID_IT_AGAIN") ; connect_2(&mut test_board, &mut score_board, &mut record, actual_player, pos2check, dir) }
                _ => { vec![] }
            };

            // tmp_result = connect_2(&mut test_board, &mut score_board, &mut record, actual_player, pos2check, 3);
            println!("DEBUT°°°DEBUG_CONNECT: len({})", tmp_result.len());
            // tmp_result.iter().for_each(|((x,y), typeOfThreat, Opp)| {  threat_board[*x][*y].push((*typeOfThreat, Opp.clone())) } );
            // ret_debug.iter().for_each(|&x| print!("{}", x)); 
            println!("DEBUG_CONNECT: len({})", tmp_result.len());

            tmp_result.iter().for_each(|(defensive_move, type_of_threat, opp)| {
                println!("-----------------");
                println!("DEFENSIVE_MOVE-TMP-RESULT:");
                println!("({},{})", defensive_move.0, defensive_move.1);
                println!("typeOfThreat:");
                match type_of_threat {
                    TypeOfThreat::FIVE => println!("FIVE"),
                    TypeOfThreat::FIVE_TAKE => println!("FIVE_TAKE"),
                    TypeOfThreat::FOUR_O => println!("FOUR_O"),
                    TypeOfThreat::FOUR_SO => println!("FOUR_SO"),
                    TypeOfThreat::TAKE => println!("TAKE"),
                    TypeOfThreat::THREE_O => println!("THREE_O"),
                }
                println!("Responses:");
                opp.iter().for_each(|(x,y)| println!("({},{})", x, y));
            });

                expected_result.iter().for_each(|(defensive_move, type_of_threat, opp)| {
                    println!("-----------------");
                    println!("DEFENSIVE_MOVE_EXPECTED:");
                    println!("({},{})", defensive_move.0, defensive_move.1);
                    println!("typeOfThreat:");
                    match type_of_threat {
                        TypeOfThreat::FIVE => println!("FIVE"),
                        TypeOfThreat::FIVE_TAKE => println!("FIVE_TAKE"),
                        TypeOfThreat::FOUR_O => println!("FOUR_O"),
                        TypeOfThreat::FOUR_SO => println!("FOUR_SO"),
                        TypeOfThreat::TAKE => println!("TAKE"),
                        TypeOfThreat::THREE_O => println!("THREE_O"),
                    }
                    println!("Responses:");
                    opp.iter().for_each(|(x,y)| println!("({},{})", x, y));
                });
                if tmp_result != vec![] {
                    break ;
                }
        }

            // println!("DEBUG_CONNECT: len({})", threat_board.len());

            // threat_board.iter().enumerate().for_each(|(i_x, x)| {
            //     println!("i_x: {}", i_x);
            //     x.iter().enumerate().for_each(|(i_y, y)| {
            //         println!("i_y: {}", i_y);
            //         println!("y_len: {}", y.len());
            //         y.iter().for_each(|(type_of_threat, opp)| {
            //             println!("-----------------");
            //             println!("DEFENSIVE_MOVE:");
            //             println!("({},{})", i_x, i_y);
            //             println!("typeOfThreat:");
            //             match type_of_threat {
            //                 TypeOfThreat::FIVE => println!("FIVE"),
            //                 TypeOfThreat::FIVE_TAKE => println!("FIVE_TAKE"),
            //                 TypeOfThreat::FOUR_O => println!("FOUR_O"),
            //                 TypeOfThreat::FOUR_SO => println!("FOUR_SO"),
            //                 TypeOfThreat::TAKE => println!("TAKE"),
            //                 TypeOfThreat::THREE_O => println!("THREE_O"),
            //             }
            //             println!("Responses:");
            //             opp.iter().for_each(|(x,y)| println!("({},{})", x, y));
            //         });
            //     });
            // });
            tmp_result == expected_result
    }

    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn threat_goood() {
        let mut black_pos = vec![(9,8),(9,7)];
        let white_pos = vec![];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((9,6), TypeOfThreat::THREE_O, vec![]),
                ((9,9), TypeOfThreat::THREE_O, vec![])
                ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
            (9, 7),
            Some(false),
            expected_result
        ))
    }

    //fn connect_2(
    //    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    //    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    //    actual_player: Option<bool>,
    //    (x, y): (usize, usize),
    //    dir: usize,
    // fn test_connect_2(
    //     white_pos: Vec<(usize, usize)>,
    //     black_pos: Vec<(usize, usize)>,
    //     actual_player: Option<bool>,
    //     (x, y): (usize, usize),
    //     dir: usize,
    // ) -> bool {
    //     let mut test_board = [[None; SIZE_BOARD]; SIZE_BOARD];
    //     let mut score_tab: [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD] =
    //         [[[(0, Some(false), Some(false)); 4]; SIZE_BOARD]; SIZE_BOARD];
    //     white_pos.iter().for_each(|&(x, y)| {
    //         test_board[x][y] = Some(true);
    //         change_score_board_add(&mut test_board, &mut score_tab, x as isize, y as isize);
    //     });
    //     black_pos.iter().for_each(|&(x, y)| {
    //         test_board[x][y] = Some(false);
    //         change_score_board_add(&mut test_board, &mut score_tab, x as isize, y as isize);
    //     });
    //     for i in 0..19 {
    //         for j in 0..19 {
    //             match test_board[j][i] {
    //                 Some(true) => print!("B"),
    //                 Some(false) => print!("N"),
    //                 None => print!("E"),
    //             }
    //             score_tab[j][i].iter().for_each(|&(value, a, b)| {
    //                 print!("{:2}{}{}", value, get_bool!(a), get_bool!(b))
    //             });
    //             print!(" ");
    //         }
    //         println!();
    //     }
    //     let res = connect_2(&mut test_board, &mut score_tab, actual_player, (x, y), dir);
    //     res.iter().for_each(|((x, y), _, answers)| {
    //         println!("danger : {}/{}", x, y);
    //         print!("Answers : ");
    //         answers
    //             .iter()
    //             .for_each(|&(ans_x, ans_y)| print!("{}/{}---", ans_x, ans_y));
    //         println!();
    //     });
    //     false
    // }

    // #[test]
    // fn connect_2_0() {
    //     let x = 8;
    //     let y = 8;
    //     let black_pos = vec![(x, y), (x + 1, y + 1)];
    //     let white_pos = vec![];
    //     let dir = 0;
    //     assert!(test_connect_2(white_pos, black_pos, Some(false), (x, y), 0))
    // }
}
