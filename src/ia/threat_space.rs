use super::super::checks::after_turn_check::DIRECTIONS;
use super::super::model::board::Board;
//use super::super::checks::capture::DIRS;
use super::super::checks::double_three::check_double_three_hint;
use super::super::render::board::SIZE_BOARD;
//use super::heuristic;
// use super::handle_board::*;
//use super::get_ia;

macro_rules! get_opp {
    ($e:expr) => {
        match $e {
            Some(a) => Some(!a),
            _ => unreachable!(),
        }
    };
}

macro_rules! valid_coord {
    (
        $x: expr,
        $y: expr
    ) => {
        $x >= 0 && $x < SIZE_BOARD as isize && $y >= 0 && $y < SIZE_BOARD as isize
    };
}

macro_rules! valid_coord_uniq {
    (
        $x: expr
    ) => {
        $x >= 0 && $x < SIZE_BOARD as isize
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
            && $board.get_pawn($new_line as usize, $new_col as usize) == $actual_player
        {
            $record[$new_line as usize][$new_col as usize][$dir] = false;
            $new_line += (DIRECTIONS[$dir].0 * $orientation);
            $new_col += (DIRECTIONS[$dir].1 * $orientation);
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
            && $board.get_pawn($new_line as usize, $new_col as usize) == $actual_player
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
        $new_line += (DIRECTIONS[$dir].0 * $orientation);
        $new_col += (DIRECTIONS[$dir].1 * $orientation);
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
//const MAX_MULTIPLE_DEFENSE_MOVES: usize = 4;

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub enum TypeOfThreat {
    // NONE,
    EMPTY = 0,
    ThreeOF = 1,
    ThreeOC = 2,
    FourSOF = 3,
    FourSOC = 4,
    FourOF = 5,
    FourOC = 6,
    FiveTake = 7,
    FourTake = 8,
    ThreeTake = 9,
    TwoTake = 10,
    OneTake = 11,
    WIN = 12,
    AlreadyWon = 13,
}

// Aim of function :
// Initialize a record for efficient tracking of modifications afterwards
// Sets adversatory and empty positions to false and the player's ones to true
// (for each direction at a given position)
pub fn initialize_record(
    board: &mut Board,
    actual_player: Option<bool>,
) -> [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD] {
    let mut record = [[[false; 4]; SIZE_BOARD]; SIZE_BOARD];

    for line in 0..SIZE_BOARD {
        for col in 0..SIZE_BOARD {
            if board.get_pawn(line, col) == actual_player {
                for dir in 0..4 {
                    record[line][col][dir] = true;
                }
            }
        }
    }
    record
}

fn capture_blank(
    board: &mut Board,
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
        let mut align = [0, 0, 0, 0];
        let mut index = 0usize;
        for way in [-1, 1].iter() {
            for step in [1, 2].iter() {
                let new_x = x as isize + way * dx * *step as isize;
                let new_y = y as isize + way * dy * *step as isize;
                match board.get(new_x as usize, new_y as usize){
                    Some(a) if a == actual_player => align[index * 2 + step - 1] = 2,
                    Some(None) => {
                        align[index * 2 + step - 1] = 1;
                        break;
                    }
                    Some(_) => {
                        align[index * 2 + step - 1] = 3;
                        break;
                    }
                    None => break,
                }
//                if !valid_coord!(new_x, new_y) {
//                    break;
//                }
//                match board[new_x as usize][new_y as usize] {
//                    None => {
//                        align[index * 2 + step - 1] = 1;
//                        break;
//                    }
//                    a if a == actual_player => align[index * 2 + step - 1] = 2,
//                    _ => {
//                        align[index * 2 + step - 1] = 3;
//                        break;
//                    }
//                }
            }
            index += 1;
        }
        if align[0] == 2 && align[1] == 3 && align[2] == 1 {
            ret.push((x + dx as usize, y + dy as usize));
        } else if align[0] == 3 && align[2] == 2 && align[3] == 1 {
            ret.push((x + 2 * dx as usize, y + 2 * dy as usize));
        } else if align[0] == 1 && align[2] == 2 && align[3] == 3 {
            ret.push(((x as isize - dx) as usize, (y as isize - dy) as usize));
        } else if align[0] == 2 && align[1] == 1 && align[2] == 3 {
            ret.push((
                (x as isize - 2 * dx) as usize,
                (y as isize - 2 * dy) as usize,
            ));
        }
    }
    ret
}

fn capture_coordinates_and_blank(
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    board: &mut Board,
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
                (0, Some(false), Some(false)) => {
                    let (mut new_line2, mut new_col2): (isize, isize) = (x as isize, y as isize);
                    let (mut new_line3, mut new_col3): (isize, isize) = (x as isize, y as isize);
                    explore_one!(new_line2, new_col2, new_dir, -1);
                    explore_one!(new_line3, new_col3, new_dir, 1);

                    let (mut new_line2, mut new_col2) =
                        match (valid_coord_uniq!(new_line2), valid_coord_uniq!(new_col2)) {
                            (true, true) => (new_line2, new_col2),
                            (false, true) => (x as isize, new_col2),
                            (true, false) => (new_line2, y as isize),
                            (false, false) => (x as isize, y as isize),
                        };

                    let (mut new_line3, mut new_col3) =
                        match (valid_coord_uniq!(new_line3), valid_coord_uniq!(new_col3)) {
                            (true, true) => (new_line3, new_col3),
                            (false, true) => (x as isize, new_col3),
                            (true, false) => (new_line3, y as isize),
                            (false, false) => (x as isize, y as isize),
                        };
                    match (
                        score_board[new_line2 as usize][new_col2 as usize][new_dir],
                        score_board[new_line3 as usize][new_col3 as usize][new_dir],
                    ) {
                        ((1, Some(true), Some(false)), (0, Some(false), Some(false))) => {
                            coordinates.push((new_line3 as usize, new_col3 as usize));
                        }
                        ((0, Some(false), Some(false)), (1, Some(false), Some(true))) => {
                            coordinates.push((new_line2 as usize, new_col2 as usize));
                        }
                        ((1, Some(false), Some(false)), (1, Some(false), Some(false))) => {
                            let opp = get_opp!(actual_player);
                            if opp == board.get_pawn(new_line2 as usize, new_col2 as usize)
                                && board.get_pawn(new_line3 as usize, new_col3 as usize) == actual_player
                            {
                                explore_one!(new_line3, new_col3, new_dir, 1);
                                coordinates.push((new_line3 as usize, new_col3 as usize));
                            } else if board.get_pawn(new_line2 as usize,new_col2 as usize) == actual_player
                                && opp == board.get_pawn(new_line3 as usize, new_col3 as usize)
                            {
                                explore_one!(new_line2, new_col2, new_dir, -1);
                                coordinates.push((new_line2 as usize, new_col2 as usize));
                            }
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        }
    }
    coordinates
}

fn capture_coordinates(
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    board: &mut Board,
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

pub fn capture_coordinates_vec(
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    board: &mut Board,
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
    board: &mut Board,
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
    let (mut ccline, mut cccol) = (*cline, *ccol);
    let mut tmp_positions: Vec<Vec<(usize, usize)>> = vec![];
    for _ in 0..limit {
        tmp_positions.push(capture_coordinates_and_blank(
            score_board,
            board,
            actual_player,
            ccline as usize,
            cccol as usize,
            dir,
        ));
        explore_one!(ccline, cccol, dir, orientation);
    }
    all_threats.push((
        (*new_line as usize, *new_col as usize),
        threat,
        flatten!(tmp_positions),
    ));
}

fn manage_so(
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    board: &mut Board,
    record: &mut [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD],
    actual_player: Option<bool>,
    dir: usize,
    mut new_line: isize,
    mut new_col: isize,
    way: isize,
    opp_way: isize,
    all_threats: &mut Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)>,
) -> () {
    explore_align!(board, record, new_line, new_col, actual_player, dir, way);
    let (mut nline, mut ncol) = (new_line, new_col);
    explore_one!(nline, ncol, dir, way);
    let (cline, ccol) = (new_line, new_col);
    if valid_coord!(nline, ncol)
        && board.get_pawn(nline as usize, ncol as usize) == actual_player
        && score_board[nline as usize][ncol as usize][dir].0 > 0
    {
        match score_board[nline as usize][ncol as usize][dir].0 {
            x if x >= 5 => {} // Instant win, no need to do manage it, we see it in the recursive!
            4 => {
                all_threats.push((
                    (new_line as usize, new_col as usize),
                    TypeOfThreat::OneTake,
                    capture_coordinates_and_blank(
                        score_board,
                        board,
                        actual_player,
                        cline as usize,
                        ccol as usize,
                        dir,
                    ),
                ));
            }
            3 => explore_and_find_threats(
                score_board,
                board,
                2,
                opp_way,
                &cline,
                &ccol,
                TypeOfThreat::TwoTake,
                actual_player,
                dir,
                all_threats,
                &new_line,
                &new_col,
            ),
            2 => explore_and_find_threats(
                score_board,
                board,
                3,
                opp_way,
                &cline,
                &ccol,
                TypeOfThreat::ThreeTake,
                actual_player,
                dir,
                all_threats,
                &new_line,
                &new_col,
            ),
            1 => explore_and_find_threats(
                score_board,
                board,
                4,
                opp_way,
                &cline,
                &ccol,
                TypeOfThreat::FourTake,
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
            TypeOfThreat::FiveTake,
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
    board: &mut Board,
    record: &mut [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD],
    actual_player: Option<bool>,
    dir: usize,
) -> Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> {
    let new_line: isize = line as isize;
    let new_col: isize = col as isize;
    let mut all_threats: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![];

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
                1,
                -1,
                &mut all_threats,
            );
            all_threats
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
                -1,
                1,
                &mut all_threats,
            );
            all_threats
        }
        (Some(false), Some(false)) => {
            let new_line2: isize = line as isize;
            let new_col2: isize = col as isize;
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
                1,
                -1,
                &mut all_threats,
            );
            all_threats
        }
        _ => all_threats,
    }
}

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

fn remove_duplicates_pos(coords: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    let mut ret = vec![];

    coords.iter().for_each(|&(x, y)| {
        let mut duplicate = false;
        for &(cmp_x, cmp_y) in ret.iter() {
            if cmp_x == x && cmp_y == y {
                duplicate = true;
                break;
            }
        }
        if !duplicate {
            ret.push((x, y));
        }
    });
    ret
}

pub fn connect_2(
    board: &mut Board,
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    record: &mut [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD],
    actual_player: Option<bool>,
    (x, y): (usize, usize),
    dir: usize,
) -> Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> {
    let mut ret: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = Vec::with_capacity(2);
    let focused_tuple = score_board[x][y][dir as usize];
    let (dx, dy) = DIRECTIONS[dir];

    record[x][y][dir] = false;
    for way in [-1, 1].iter() {
        let mut new_x = x as isize;
        let mut new_y = y as isize;
        explore_align!(board, record, new_x, new_y, actual_player, dir, way);
        let (actual_edge, opp_edge): (Option<bool>, Option<bool>) = get_edges!(way, focused_tuple);
        if actual_edge != Some(false) {
            continue;
        }
        let mut space = 1;
        let mut align_ally = 0;
        // explore_align_light!(board, new_x, new_y, actual_player, dir, way);
        let mut cursor_x = new_x;
        let mut cursor_y = new_y;
        loop {
            cursor_x += dx * way;
            cursor_y += dy * way;
            if space >= 3 || align_ally != 0{
                break;
            }
            match board.get(cursor_x as usize, cursor_y as usize){
                Some(a) if a == actual_player =>{
                    align_ally = score_board[cursor_x as usize][cursor_y as usize][dir].0
                }
                Some(None) => space += 1,
                Some(_) => break,
                None => break,
            }
//            if !valid_coord!(cursor_x, cursor_y) || space >= 3 || align_ally != 0 {
//                break;
//            }
//            match board[cursor_x as usize][cursor_y as usize] {
//                None => space += 1,
//                a if a == actual_player => {
//                    align_ally = score_board[cursor_x as usize][cursor_y as usize][dir].0
//                }
//                _ => break,
//            }
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
                        let mut ally_edge: Option<bool> = None;
                        if valid_coord!((new_x + way * dx), (new_y + way * dy)) {
                            let (tmp_edge, _) = get_edges!(
                                way,
                                score_board[(new_x + way * dx) as usize]
                                    [(new_y + way * dy) as usize][dir]
                            );
                            ally_edge = tmp_edge;
                        }
                        let steps: Vec<isize> = vec![-2, -1, 1];
                        let (opp_steps, threat) = match ally_edge {
                            Some(false) => (vec![], TypeOfThreat::FourOC),
                            _ => (vec![-3], TypeOfThreat::FourSOC),
                        };
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            threat,
                            create_align(steps, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    2 => {
                        // _00_00#
                        //opened edge, 1 space, 2 more pawn
                        let steps = vec![-2, -1, 1, 2];
                        let (opp_steps, threat) = (vec![], TypeOfThreat::FiveTake);
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            threat,
                            create_align(steps, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    3 => {
                        // _00_000#
                        //opened edge, 1 space, 3 more pawn
                        let steps = vec![-1, 1, 2];
                        let (opp_steps, threat) = (vec![], TypeOfThreat::FourTake);
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            threat,
                            create_align(steps, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    4 => {
                        // _00_0000#
                        //opened edge, 1 space, 4 more pawn
                        let steps = vec![1, 2];
                        let (opp_steps, threat) = (vec![], TypeOfThreat::ThreeTake);
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

                        // let (mut tmp_x, mut tmp_y) = (x as isize, y as isize);
                        // explore_align_light!(board, tmp_x, tmp_y, actual_player, dir, -way);
                        // tmp_x -= way * dx;
                        // tmp_y -= way * dy;
                        // if !valid_coord!(tmp_x, tmp_y)
                        //     || board[tmp_x as usize][tmp_y as usize] != actual_player
                        // {
                        //     ();
                        // } else {
                        let steps = vec![-2, -1];
                        let opp_steps = vec![-3, 1];
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            TypeOfThreat::ThreeOC,
                            create_align(steps, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps, *way, (new_x, new_y), (dx, dy)),
                        ));
                        // }
                    }
                    1 => {
                        // _00__0#
                        //opened edge, 2 space, 1 more pawn
                        let steps_no_space = vec![-2, -1, 2];
                        let opp_steps_no_space = vec![-3, 1];
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            TypeOfThreat::ThreeOC,
                            create_align(steps_no_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_no_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                        let steps_with_space = vec![-2, -1, 2];
                        let opp_steps_with_space = vec![0];
                        gather_infos.push((
                            ((new_x + way * dx) as usize, (new_y + way * dy) as usize),
                            TypeOfThreat::FourSOF,
                            create_align(steps_with_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_with_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    2 => {
                        // _00__00#
                        //opened edge, 2 space, 2 more pawn
                        let steps_no_space = vec![-1, 2];
                        let opp_steps_no_space = vec![-3, 1];
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            TypeOfThreat::ThreeOC,
                            create_align(steps_no_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_no_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                        let steps_with_space = vec![-1, 2];
                        let opp_steps_with_space = vec![0];
                        gather_infos.push((
                            ((new_x + way * dx) as usize, (new_y + way * dy) as usize),
                            TypeOfThreat::FourSOF,
                            create_align(steps_with_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_with_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    3 => {
                        // _00__000#
                        //opened edge, 2 space, 3 more pawn
                        let steps_no_space = vec![2];
                        let opp_steps_no_space = vec![-3, 1];
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            TypeOfThreat::ThreeOC,
                            create_align(steps_no_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_no_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                        let steps_with_space = vec![2];
                        let opp_steps_with_space = vec![0];
                        gather_infos.push((
                            ((new_x + way * dx) as usize, (new_y + way * dy) as usize),
                            TypeOfThreat::FourSOC,
                            create_align(steps_with_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_with_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    4 => {
                        // _00__0000#
                        //opened edge, 2 space, 4 more pawn
                        let steps_no_space = vec![2];
                        let opp_steps_no_space = vec![-3, 1];
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            TypeOfThreat::ThreeOC,
                            create_align(steps_no_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_no_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                        let steps_with_space = vec![2];
                        let opp_steps_with_space = vec![0];
                        gather_infos.push((
                            ((new_x + way * dx) as usize, (new_y + way * dy) as usize),
                            TypeOfThreat::FiveTake,
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
                        TypeOfThreat::ThreeOC,
                        create_align(steps_no_space, *way, (new_x, new_y), (dx, dy)),
                        create_align(opp_steps_no_space, *way, (new_x, new_y), (dx, dy)),
                    ));
                    let steps_with_space = vec![-2, -1];
                    let opp_steps_with_space = vec![0, -3, 2];
                    gather_infos.push((
                        ((new_x + way * dx) as usize, (new_y + way * dy) as usize),
                        TypeOfThreat::ThreeOF,
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
                        let mut ally_edge: Option<bool> = None;
                        if valid_coord!((new_x + way * dx), (new_y + way * dy)) {
                            let (tmp_edge, _) = get_edges!(
                                way,
                                score_board[(new_x + way * dx) as usize]
                                    [(new_y + way * dy) as usize][dir]
                            );
                            ally_edge = tmp_edge;
                        }
                        if ally_edge == Some(false) {
                            let steps: Vec<isize> = vec![-2, -1, 1];
                            let opp_steps = vec![2];
                            gather_infos.push((
                                (new_x as usize, new_y as usize),
                                TypeOfThreat::FourSOC,
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
                        let (opp_steps, threat) = (vec![], TypeOfThreat::FiveTake);
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
                        let (opp_steps, threat) = (vec![], TypeOfThreat::FourTake);
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            threat,
                            create_align(steps, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    4 => {
                        // #00_0000?
                        let steps = vec![1, 2];
                        let (opp_steps, threat) = (vec![], TypeOfThreat::ThreeTake);
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
                            TypeOfThreat::FourSOF,
                            create_align(steps_no_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_no_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                        let steps_with_space = vec![-2, -1, 2];
                        let opp_steps_with_space = vec![0];
                        gather_infos.push((
                            ((new_x + way * dx) as usize, (new_y + way * dy) as usize),
                            TypeOfThreat::FourSOF,
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
                            TypeOfThreat::ThreeOF,
                            create_align(steps_no_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_no_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                        let steps_with_space = vec![-1, 2];
                        let opp_steps_with_space = vec![0];
                        gather_infos.push((
                            ((new_x + way * dx) as usize, (new_y + way * dy) as usize),
                            TypeOfThreat::ThreeOF,
                            create_align(steps_with_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_with_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    3 => {
                        // #00__000?
                        let steps_no_space = vec![2];
                        let opp_steps_no_space = vec![1];
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            TypeOfThreat::ThreeOF,
                            create_align(steps_no_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_no_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                        let steps_with_space = vec![2];
                        let opp_steps_with_space = vec![0];
                        gather_infos.push((
                            ((new_x + way * dx) as usize, (new_y + way * dy) as usize),
                            TypeOfThreat::FourSOC,
                            create_align(steps_with_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_with_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    4 => {
                        // #00__0000?
                        let steps_no_space = vec![2];
                        let opp_steps_no_space = vec![1];
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            TypeOfThreat::ThreeOF,
                            create_align(steps_no_space, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps_no_space, *way, (new_x, new_y), (dx, dy)),
                        ));
                        let steps_with_space = vec![2];
                        let opp_steps_with_space = vec![0];
                        gather_infos.push((
                            ((new_x + way * dx) as usize, (new_y + way * dy) as usize),
                            TypeOfThreat::FiveTake,
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
        gather_infos
            .iter()
            .for_each(|(pos, threat, alignement, opp_pos)| {
                let mut answers: Vec<(usize, usize)> =
                    capture_blank(board, actual_player, pos.0, pos.1, dir);
                opp_pos.iter().for_each(|&coord| answers.push(coord));
                alignement.iter().for_each(|&(align_x, align_y)| {
                    let captures = capture_coordinates(
                        score_board,
                        board,
                        actual_player,
                        align_x,
                        align_y,
                        dir,
                    );
                    captures.iter().for_each(|&tuple| answers.push(tuple));
                });
                ret.push((*pos, *threat, remove_duplicates_pos(answers)));
            });
    }
    ret
}

pub fn connect_3(
    board: &mut Board,
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    record: &mut [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD],
    actual_player: Option<bool>,
    (x, y): (usize, usize),
    dir: usize,
) -> Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> {
    let mut ret: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = Vec::with_capacity(2);
    let focused_tuple = score_board[x][y][dir as usize];
    let (dx, dy) = DIRECTIONS[dir];

    record[x][y][dir] = false;
    for way in [-1, 1].iter() {
        let mut space = 1;
        let mut align_ally = 0;
        let mut new_x = x as isize;
        let mut new_y = y as isize;
        explore_align!(board, record, new_x, new_y, actual_player, dir, way);
        let (actual_edge, opp_edge): (Option<bool>, Option<bool>) = get_edges!(way, focused_tuple);
        if actual_edge != Some(false) {
            continue;
        }
        // explore_align_light!(board, new_x, new_y, actual_player, dir, way);
        let mut cursor_x = new_x;
        let mut cursor_y = new_y;
        loop {
            cursor_x += dx * way;
            cursor_y += dy * way;
            if space >= 2 || align_ally != 0{
                break;
            }
            match board.get(cursor_x as usize, cursor_y as usize){
                Some(a) if a == actual_player =>{
                    align_ally = score_board[cursor_x as usize][cursor_y as usize][dir].0
                }
                Some(None) => space += 1,
                Some(_) => break,
                None => break,
            }
//            if !valid_coord!(cursor_x, cursor_y) || space >= 2 || align_ally != 0 {
//                break;
//            }
//            match board[cursor_x as usize][cursor_y as usize] {
//                None => space += 1,
//                a if a == actual_player => {
//                    align_ally = score_board[cursor_x as usize][cursor_y as usize][dir].0
//                }
//                _ => break,
//            }
        }
        let mut gather_infos: Vec<(
            (usize, usize),
            TypeOfThreat,
            Vec<(usize, usize)>,
            Vec<(usize, usize)>,
        )> = Vec::with_capacity(2);
        //println!("space : {}\talign : {}", space, align_ally);

        match opp_edge {
            Some(false) => match space {
                1 => match align_ally {
                    0 => {
                        // _000_?
                        let steps = vec![-3, -2, -1];
                        let (opp_steps, threat) = (vec![-4], TypeOfThreat::FourSOC);
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            threat,
                            create_align(steps, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    1 => {
                        // _000_0?
                        //opened edge, 1 space, 1 more pawn
                        let steps = vec![-3, -2, -1, 1];
                        let (opp_steps, threat) = (vec![], TypeOfThreat::FiveTake);
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            threat,
                            create_align(steps, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    2 => {
                        // _000_00?
                        //opened edge, 1 space, 2 more pawn
                        let steps = vec![-2, -1, 1];
                        let (opp_steps, threat) = (vec![], TypeOfThreat::FourTake);
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            threat,
                            create_align(steps, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    3 => {
                        // _000_000?
                        //opened edge, 1 space, 3 more pawn
                        let steps = vec![-1, 1];
                        let (opp_steps, threat) = (vec![], TypeOfThreat::ThreeTake);
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            threat,
                            create_align(steps, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    4 => {
                        // _000_0000?
                        //opened edge, 1 space, 4 more pawn
                        let steps = vec![1];
                        let (opp_steps, threat) = (vec![], TypeOfThreat::TwoTake);
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            threat,
                            create_align(steps, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    5..=9 => (),
                    _ => unreachable!(),
                },
                2 => {
                    // _000__#
                    //opened edge, 2 space, 0 more pawn
                    let steps_no_space = vec![-3, -2, -1];
                    let opp_steps_no_space = vec![];
                    gather_infos.push((
                        (new_x as usize, new_y as usize),
                        TypeOfThreat::FourOC,
                        create_align(steps_no_space, *way, (new_x, new_y), (dx, dy)),
                        create_align(opp_steps_no_space, *way, (new_x, new_y), (dx, dy)),
                    ));
                    let steps_with_space = vec![-3, -2, -1];
                    let opp_steps_with_space = vec![0];
                    gather_infos.push((
                        ((new_x + way * dx) as usize, (new_y + way * dy) as usize),
                        TypeOfThreat::FourSOF,
                        create_align(steps_with_space, *way, (new_x, new_y), (dx, dy)),
                        create_align(opp_steps_with_space, *way, (new_x, new_y), (dx, dy)),
                    ));
                }
                _ => unreachable!(),
            },
            _ => match space {
                1 => match align_ally {
                    1 => {
                        // #000_0?
                        let steps: Vec<isize> = vec![-3, -2, -1, 1];
                        let opp_steps = vec![];
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            TypeOfThreat::FiveTake,
                            create_align(steps, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    2 => {
                        // #000_00?
                        let steps = vec![-2, -1, 1];
                        let (opp_steps, threat) = (vec![], TypeOfThreat::FourTake);
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            threat,
                            create_align(steps, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    3 => {
                        // #000_000?
                        let steps = vec![-1, 1];
                        let (opp_steps, threat) = (vec![], TypeOfThreat::ThreeTake);
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            threat,
                            create_align(steps, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    4 => {
                        // #000_0000?
                        let steps = vec![1];
                        let (opp_steps, threat) = (vec![], TypeOfThreat::TwoTake);
                        gather_infos.push((
                            (new_x as usize, new_y as usize),
                            threat,
                            create_align(steps, *way, (new_x, new_y), (dx, dy)),
                            create_align(opp_steps, *way, (new_x, new_y), (dx, dy)),
                        ));
                    }
                    0 => (),
                    // #000_#
                    5 => (),
                    _ => unreachable!(),
                },
                2 => {
                    // #000__?
                    let steps_no_space = vec![-3, -2, -1];
                    let opp_steps_no_space = vec![1];
                    gather_infos.push((
                        (new_x as usize, new_y as usize),
                        TypeOfThreat::FourSOC,
                        create_align(steps_no_space, *way, (new_x, new_y), (dx, dy)),
                        create_align(opp_steps_no_space, *way, (new_x, new_y), (dx, dy)),
                    ));
                    let steps_with_space = vec![-3, -2, -1];
                    let opp_steps_with_space = vec![0];
                    gather_infos.push((
                        ((new_x + way * dx) as usize, (new_y + way * dy) as usize),
                        TypeOfThreat::FourSOF,
                        create_align(steps_with_space, *way, (new_x, new_y), (dx, dy)),
                        create_align(opp_steps_with_space, *way, (new_x, new_y), (dx, dy)),
                    ));
                }
                _ => unreachable!(),
            },
        };
        gather_infos
            .iter()
            .for_each(|(pos, threat, alignement, opp_pos)| {
                let mut answers: Vec<(usize, usize)> =
                    capture_blank(board, actual_player, pos.0, pos.1, dir);
                opp_pos.iter().for_each(|&coord| answers.push(coord));
                alignement.iter().for_each(|&(align_x, align_y)| {
                    let captures = capture_coordinates(
                        score_board,
                        board,
                        actual_player,
                        align_x,
                        align_y,
                        dir,
                    );
                    captures.iter().for_each(|&tuple| answers.push(tuple));
                });
                ret.push((*pos, *threat, remove_duplicates_pos(answers)));
            });
    }
    ret
}

pub fn threat_search_space(
    board: &mut Board,
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    actual_player: Option<bool>,
    actual_take: &mut isize,
) -> Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> {
    // 1. Initialize datastructures storing ready to be checked positions as well as threats
    let mut record: [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD] = initialize_record(board, actual_player);

    // 1.1. Initialize Threat board -> Vec containing with_capacity data (3 avrg max_possible threats per position) | (4 max defensive)
    let mut threat_board: Vec<Vec<(TypeOfThreat, Vec<(usize, usize)>)>> = (0..SIZE_BOARD)
        .map(|_| {
            (0..SIZE_BOARD)
                .map(|_| {
                    (
                        TypeOfThreat::EMPTY,
                        Vec::with_capacity(AVRG_MAX_MULTIPLE_THREATS),
                    )
                })
                .collect()
        })
        .collect();

    let mut catch_board: [[u8; SIZE_BOARD]; SIZE_BOARD] = [[0; SIZE_BOARD]; SIZE_BOARD];

    // 2. Parse board for actual_player's pawns
    for line in 0..SIZE_BOARD {
        for col in 0..SIZE_BOARD {
            if board.get_pawn(line, col) == actual_player {
                for dir in 0..4 {
                    if record[line][col][dir] {
                        let ret: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
                            match score_board[line][col][dir].0 {
                                5..=9 => vec![((line, col), TypeOfThreat::AlreadyWon, vec![])],

                                4 => connect_4(
                                    (line, col),
                                    score_board,
                                    board,
                                    &mut record,
                                    actual_player,
                                    dir,
                                ),
                                3 => connect_3(
                                    board,
                                    score_board,
                                    &mut record,
                                    actual_player,
                                    (line, col),
                                    dir,
                                ),
                                // 2 => continue,
                                2 => connect_2(
                                   board,
                                   score_board,
                                   &mut record,
                                   actual_player,
                                   (line, col),
                                   dir,
                                ),
                                1 => continue,
                                _ => {
                                    board.print();

                                    unreachable!()
                                }
                            };
                        //println!("len threat : {}", ret.len());
                        //ret.iter()
                        //    .for_each(|((x, y), _, _)| println!("threat : {}:{}", x, y));
                        if ret.len() >= 1 && ret[0].1 >= TypeOfThreat::WIN {
                            return ret;
                        }
                        //                        println!("nbr threats : {} from : ({},{})", ret.len(), line, col);
                        ret.iter()
                            .filter(|((x, y), _, _)| {
                                !check_double_three_hint(
                                    board,
                                    actual_player,
                                    *x as isize,
                                    *y as isize,
                                )
                            })
                            .for_each(|((x, y), typeofthreat, opp)| {
                                if threat_board[*x][*y].0 < *typeofthreat {
                                    threat_board[*x][*y].0 = *typeofthreat;
                                }
                                opp.iter().for_each(|&el| threat_board[*x][*y].1.push(el));
                            });
                    }
                }
            } else if board.get_pawn(line, col) == get_opp!(actual_player) {
                for dir in 0..4 {
                    let (mut new_line, mut new_col): (isize, isize) = (line as isize, col as isize);
                    match score_board[line][col][dir] {
                        (2, Some(true), Some(false)) => {
                            // println!("ici");
                            explore_align_light!(
                                board,
                                new_line,
                                new_col,
                                get_opp!(actual_player),
                                dir,
                                1
                            );
                            if !check_double_three_hint(board, actual_player, new_line, new_col) {
                                catch_board[new_line as usize][new_col as usize] += 1;
                            }
                        }
                        // (2, Some(false), Some(true)) | (2, Some(false), None) => {
                        (2, Some(false), Some(true)) => {
                            // println!("la");
                            explore_align_light!(
                                board,
                                new_line,
                                new_col,
                                get_opp!(actual_player),
                                dir,
                                -1
                            );
                            if !check_double_three_hint(board, actual_player, new_line, new_col) {
                                catch_board[new_line as usize][new_col as usize] += 1;
                            }
                        }
                        _ => continue,
                    }
                }
            }
        }
    }
    // Check for WIN with catches
    // 1. Find max and count how many different catches we have
    let mut max: ((usize, usize), u8) = ((0, 0), 0);
    let mut counter: u8 = 0;
    catch_board.iter().enumerate().for_each(|(line, elements)| {
        elements.iter().enumerate().for_each(|(col, &el)| {
            if el > 0 {
                counter += 1;
                if el > max.1 {
                    max = ((line, col), el);
                }
            }
        });
    });

    // 2. Check if catch leads to instant win and returns it yes
    if *actual_take + (max.1 / 2) as isize >= 5
        || *actual_take + (max.1 / 2) as isize == 4 && counter > 2
    {
        return vec![(max.0, TypeOfThreat::WIN, vec![])];
    }

    // // Check for win with gameplay
    // // 1. Construct the returned datastruct
    let mut result = vec![];
    for line in 0..SIZE_BOARD {
        //        print!("//");
        for col in 0..SIZE_BOARD {
            // let (threat, answers) = threat_board[line][col];
            // let toto = threat_board[line][col].1;
            //print!(
            //    "{}  ",
            //    match threat_board[line][col].0 {
            //        TypeOfThreat::FiveTake => "FiveTake",
            //        TypeOfThreat::FourTake => "FourTake",
            //        TypeOfThreat::ThreeTake => "ThreeTake",
            //        TypeOfThreat::TwoTake => "TwoTake",
            //        TypeOfThreat::OneTake => "OneTake",
            //        TypeOfThreat::FourO => "FourO",
            //        TypeOfThreat::FourSOF => "FourSOF",
            //        TypeOfThreat::ThreeOF => "ThreeOF",
            //        TypeOfThreat::WIN => "WIN",
            //        TypeOfThreat::EMPTY => "EMPTY",
            //    }
            //);

            if threat_board[line][col].0 != TypeOfThreat::EMPTY {
                //        println!("add threat");
                if threat_board[line][col].0 >= TypeOfThreat::FiveTake {
                    return vec![((line, col), TypeOfThreat::WIN, vec![])];
                }
                result.push((
                    (line, col),
                    threat_board[line][col].0,
                    threat_board[line][col].1.iter().cloned().collect(),
                ));
            }
        }
        //       println!();
    }
    //println!("//all threats :");
    //result.iter().for_each(|((x, y), _, answers)| {
    //    println!("//{}:{}", x, y);
    //});
    // Sort by threat in descending order
    result.sort_by(|(_, threat_a, _), (_, threat_b, _)| threat_b.partial_cmp(threat_a).unwrap());

    result
}

#[cfg(test)]
mod tests {
    use super::super::handle_board::change_score_board_add;
    use super::super::heuristic;
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
        let mut test_board:Board = [[None; SIZE_BOARD]; SIZE_BOARD].into();
        let mut score_tab: [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD] =
            [[[(0, Some(false), Some(false)); 4]; SIZE_BOARD]; SIZE_BOARD];
        white_pos.iter().for_each(|&(x, y)| {
            test_board.set(x, y, Some(true));
            change_score_board_add(
                &mut test_board,
                &mut score_tab,
                x,
                y,
                Some(true),
            );
        });
        black_pos.iter().for_each(|&(x, y)| {
            test_board.set(x, y, Some(false));
            change_score_board_add(
                &mut test_board,
                &mut score_tab,
                x,
                y,
                Some(false),
            );
        });
        for i in 0..19 {
            for j in 0..19 {
                match test_board.get_pawn(j, i) {
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
        let ret = capture_blank(&mut test_board, actual_player, x, y, 0);
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

    fn test_catch_board(
        board: &mut Board,
        score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
        actual_player: Option<bool>,
        actual_take: &mut isize,
    ) -> (((usize, usize), u8), bool) {
        let mut catch_board: [[u8; SIZE_BOARD]; SIZE_BOARD] = [[0; SIZE_BOARD]; SIZE_BOARD];

        for line in 0..SIZE_BOARD {
            for col in 0..SIZE_BOARD {
                if board.get_pawn(line, col) == get_opp!(actual_player) {
                    for dir in 0..4 {
                        let (mut new_line, mut new_col): (isize, isize) =
                            (line as isize, col as isize);
                        match score_board[line][col][dir] {
                            (2, Some(true), Some(false)) => {
                                println!("PREV_MATCH: ({},{})", new_line, new_col);
                                // println!("ici");
                                explore_align_light!(
                                    board,
                                    new_line,
                                    new_col,
                                    get_opp!(actual_player),
                                    dir,
                                    1
                                );
                                // explore_one!(new_line, new_col, dir, 1);
                                // if true {
                                if !check_double_three_hint(board, actual_player, new_line, new_col)
                                {
                                    println!("I MATCH1: ({},{})", new_line, new_col);
                                    catch_board[new_line as usize][new_col as usize] += 1;
                                }
                            }
                            // (2, Some(false), Some(true)) | (2, Some(false), None) => {
                            (2, Some(false), Some(true)) => {
                                println!("PREV_MATCH: ({},{})", new_line, new_col);
                                // println!("la");
                                explore_align_light!(
                                    board,
                                    new_line,
                                    new_col,
                                    get_opp!(actual_player),
                                    dir,
                                    -1
                                );
                                // explore_one!(new_line, new_col, dir, -1);
                                // explore_one!(new_line, new_col, dir, -1);
                                if !check_double_three_hint(board, actual_player, new_line, new_col)
                                {
                                    println!("I MATCH2: ({},{})", new_line, new_col);
                                    catch_board[new_line as usize][new_col as usize] += 1;
                                }
                            }
                            _ => continue,
                        }
                    }
                }
            }
        }

        for line in 0..SIZE_BOARD {
            for col in 0..SIZE_BOARD {
                print!("{} ", catch_board[col][line]);
            }
            println!("");
        }

        // Check for WIN with catches
        // 1. Find max and count how many different catches we have
        let mut max: ((usize, usize), u8) = ((0, 0), 0);
        let mut counter: u8 = 0;
        catch_board.iter().enumerate().for_each(|(line, elements)| {
            elements.iter().enumerate().for_each(|(col, &el)| {
                if el > 0 {
                    counter += 1;
                    if el > max.1 {
                        println!("good");
                        max = ((line, col), el);
                    }
                }
            });
        });

        let decompos = max.0;
        println!(
            "Counter: {}, [({},{}),{}]",
            counter, decompos.0, decompos.1, max.1
        );

        // 2. Check if catch leads to instant win and returns it yes
        if *actual_take + (max.1 / 2) as isize >= 5
            || *actual_take + (max.1 / 2) as isize == 4 && counter > 2
        {
            println!("WIN");
            return (max, true);
        }

        (max, false)
    }

    fn test_catch_tss(
        white_pos: Vec<(usize, usize)>,
        black_pos: Vec<(usize, usize)>,
        actual_take: &mut isize,
        actual_player: Option<bool>,
        expected_result: (((usize, usize), u8), bool),
    ) -> bool {
        let mut test_bboard = [[None; SIZE_BOARD]; SIZE_BOARD];
        white_pos
            .iter()
            .for_each(|&(x, y)| test_bboard[x][y] = Some(true));
        black_pos
            .iter()
            .for_each(|&(x, y)| test_bboard[x][y] = Some(false));
        // Print initial configuration
        let mut test_board: Board = test_bboard.into();
        println!("// Initial configuration:");
        test_board.print();
//        for i in 0..19 {
//            print!("// ");
//            for j in 0..19 {
//                match test_board[j][i] {
//                    Some(true) => print!("⊖"),
//                    Some(false) => print!("⊕"),
//                    None => print!("_"),
//                }
//            }
//            println!();
//        }
        let mut score_board = heuristic::evaluate_board(&mut test_board);

        let ret = test_catch_board(
            &mut test_board,
            &mut score_board,
            actual_player,
            actual_take,
        );
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
    fn threat_catch_tss_1() {
        let black_pos = vec![(9, 8), (9, 7)];
        let white_pos = vec![];
        let mut white_take = 0_isize;
        let expected_result = (((0, 0), 0), false);

        assert!(test_catch_tss(
            white_pos,
            black_pos,
            &mut white_take,
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊖_________
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
    #[test]
    fn threat_catch_tss_2() {
        let black_pos = vec![(9, 8), (9, 7)];
        let white_pos = vec![(9, 6)];
        let mut white_take = 0_isize;
        let expected_result = (((0, 0), 0), false);

        assert!(test_catch_tss(
            white_pos,
            black_pos,
            &mut white_take,
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊖_________
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
    #[test]
    fn threat_catch_tss_3() {
        let black_pos = vec![(9, 8), (9, 7)];
        let white_pos = vec![(9, 6)];
        let mut white_take = 4_isize;
        let expected_result = (((9, 9), 2), true);

        assert!(test_catch_tss(
            white_pos,
            black_pos,
            &mut white_take,
            Some(true),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊖_________
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
    #[test]
    fn threat_catch_tss_4() {
        let black_pos = vec![(9, 8), (9, 7)];
        let white_pos = vec![(9, 6)];
        let mut white_take = 3_isize;
        let expected_result = (((9, 9), 2), false);

        assert!(test_catch_tss(
            white_pos,
            black_pos,
            &mut white_take,
            Some(true),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊖_________
    // _________⊕_________
    // _________⊕_________
    // __________⊖⊖_______
    // __________⊖________
    // ___________⊖_______
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn threat_catch_tss_5() {
        let black_pos = vec![(9, 8), (9, 7)];
        let white_pos = vec![(9, 6), (10, 9), (11, 9), (10, 10), (11, 11)];
        let mut white_take = 4_isize;
        let expected_result = (((0, 0), 0), false);

        assert!(test_catch_tss(
            white_pos,
            black_pos,
            &mut white_take,
            Some(true),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊖_________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // __________⊕________
    // ___________⊕_______
    // ____________⊖______
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn threat_catch_tss_6() {
        let black_pos = vec![(9, 8), (9, 7), (10, 10), (11, 11)];
        let white_pos = vec![(9, 6), (12, 12)];
        let mut white_take = 3_isize;
        let expected_result = (((9, 9), 4), true);

        assert!(test_catch_tss(
            white_pos,
            black_pos,
            &mut white_take,
            Some(true),
            expected_result
        ))
    }

    fn test_threat_2(
        white_pos: Vec<(usize, usize)>,
        black_pos: Vec<(usize, usize)>,
        pos2check: (usize, usize),
        actual_player: Option<bool>,
        expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)>,
    ) -> bool {
        let mut test_bboard = [[None; SIZE_BOARD]; SIZE_BOARD];
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
//        for i in 0..19 {
//            print!("// ");
//            for j in 0..19 {
//                match test_board[j][i] {
//                    Some(true) => print!("⊖"),
//                    Some(false) => print!("⊕"),
//                    None => print!("_"),
//                }
//            }
//            println!();
//        }
        let mut score_board = heuristic::evaluate_board(&mut test_board);
        let mut record: [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD] =
            initialize_record(&mut test_board, actual_player);

        let mut global_results: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![];

        for dir in 0..4 {
            let tmp_result = match score_board[pos2check.0][pos2check.1][dir].0 {
                // 4 => (),
                // 3 => { connect_4(pos2check, &mut score_board, &mut test_board, &mut record, actual_player, actual_take, dir) },
                // 2 => { println!("OUPS_I_DID_IT_AGAIN") ; connect_2(&mut test_board, &mut score_board, &mut record, actual_player, pos2check, dir) }
                2 => connect_2(
                    &mut test_board,
                    &mut score_board,
                    &mut record,
                    actual_player,
                    pos2check,
                    dir,
                ),
                _ => vec![],
            };
            if tmp_result.len() == 0 {
                continue;
            }
            // tmp_result = connect_2(&mut test_board, &mut score_board, &mut record, actual_player, pos2check, 3);
            // println!("DEBUT°°°DEBUG_CONNECT: len({})", tmp_result.len());
            // tmp_result.iter().for_each(|((x,y), typeOfThreat, Opp)| {  threat_board[*x][*y].push((*typeOfThreat, Opp.clone())) } );
            // ret_debug.iter().for_each(|&x| print!("{}", x));
            // println!("DEBUG_CONNECT: len({})", tmp_result.len());
            tmp_result
                .iter()
                .for_each(|(defensive_move, type_of_threat, opp)| {
                    global_results.push((*defensive_move, *type_of_threat, (*opp).clone()));
                    // println!("-----------------");
                    // For each result, print the details of the threat + possible response
                    println!("\n// Details: [dir:{}]", dir);
                    for i in 0..19 {
                        print!("// ");
                        for j in 0..19 {
                            // Print specific attack move
                            if (defensive_move.0, defensive_move.1) == (j as usize, i as usize) {
                                print!("⊛")
                            } else if opp.contains(&(j, i)) {
                                print!("⊙")
                            } else {
                                match test_board.get_pawn(j, i) {
                                    Some(true) => print!("⊖"),
                                    Some(false) => print!("⊕"),
                                    None => print!("_"),
                                }
                            }
                        }
                        println!();
                    }
                    // ((9,4), TypeOfThreat::FourOF, vec![(10,8)]),
                    println!("// DEFENSIVE_MOVE:");

                    print!("// (({},{}), ", defensive_move.0, defensive_move.1);
                    print!(
                        "TypeOfThreat::{}, vec![",
                        match type_of_threat {
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
                    opp.iter().enumerate().for_each(|(i, (x, y))| {
                        if i == (opp.len() - 1) {
                            print!("({},{})", x, y)
                        } else {
                            print!("({},{}),", x, y)
                        }
                    });
                    println!("])");
                });
        }

        // print expected datastruct in test
        println!();
        global_results
            .iter()
            .enumerate()
            .for_each(|(j, (defensive_move, type_of_threat, opp))| {
                print!("(({},{}), ", defensive_move.0, defensive_move.1);
                print!(
                    "TypeOfThreat::{}, vec![",
                    match type_of_threat {
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
                opp.iter().enumerate().for_each(|(i, (x, y))| {
                    if i == (opp.len() - 1) {
                        print!("({},{})", x, y)
                    } else {
                        print!("({},{}),", x, y)
                    }
                });
                if j == (global_results.len() - 1) {
                    println!("])");
                } else {
                    println!("]),");
                }
            });
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
        //                 TypeOfThreat::FiveTake => println!("FiveTake"),
        //                 TypeOfThreat::FourOF => println!("FourOF"),
        //                 TypeOfThreat::FourSOF => println!("FourSOF"),
        //                 TypeOfThreat::TAKE => println!("TAKE"),
        //                 TypeOfThreat::ThreeOF => println!("ThreeOF"),
        //             }
        //             println!("Responses:");
        //             opp.iter().for_each(|(x,y)| println!("({},{})", x, y));
        //         });
        //     });
        // });
        global_results == expected_result
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

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊕_________
    // _________⊕_________
    // _________⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,6), TypeOfThreat::ThreeOC, vec![(9,9),(9,5)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊛_________
    // _________⊙_________
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
    // DEFENSIVE_MOVE:
    // ((9,5), TypeOfThreat::ThreeOF, vec![(9,6)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // _________⊛_________
    // _________⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::ThreeOC, vec![(9,6),(9,10)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _________⊙_________
    // _________⊛_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,10), TypeOfThreat::ThreeOF, vec![(9,9)])
    #[test]
    fn threat_connect_2_normal_0() {
        let black_pos = vec![(9, 8), (9, 7)];
        let white_pos = vec![];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 6), TypeOfThreat::ThreeOC, vec![(9, 9), (9, 5)]),
            ((9, 5), TypeOfThreat::ThreeOF, vec![(9, 6), (9, 9), (9, 4)]),
            ((9, 9), TypeOfThreat::ThreeOC, vec![(9, 6), (9, 10)]),
            (
                (9, 10),
                TypeOfThreat::ThreeOF,
                vec![(9, 9), (9, 6), (9, 11)],
            ),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
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
    // ___________________
    // _________⊖_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊕_________
    // _________⊕_________
    // _________⊙_________
    // _________⊖_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,6), TypeOfThreat::ThreeOC, vec![(9,9),(9,5)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // _________⊙_________
    // _________⊖_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,5), TypeOfThreat::ThreeOF, vec![(9,6),(9,9),(9,4)])
    #[test]
    fn threat_connect_2_normal_blocked() {
        let black_pos = vec![(9, 8), (9, 7)];
        let white_pos = vec![(9, 10)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 6), TypeOfThreat::ThreeOC, vec![(9, 9), (9, 5)]),
            ((9, 5), TypeOfThreat::ThreeOF, vec![(9, 6), (9, 9), (9, 4)]),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
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
    // _______⊖⊕⊕_________
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

    // Details: [dir:2]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __________⊙________
    // _________⊕_________
    // _______⊖⊕⊕⊙________
    // _______⊛___________
    // ______⊙____________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((7,9), TypeOfThreat::ThreeOC, vec![(10,8),(10,6),(6,10)])

    // Details: [dir:2]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __________⊙________
    // _________⊕_________
    // _______⊖⊕⊕⊙________
    // _______⊙___________
    // ______⊛____________
    // _____⊙_____________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((6,10), TypeOfThreat::ThreeOF, vec![(10,8),(7,9),(10,6),(5,11)])

    // Details: [dir:2]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________⊙_______
    // __________⊛________
    // _________⊕_________
    // _______⊖⊕⊕⊙________
    // _______⊙___________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((10,6), TypeOfThreat::ThreeOC, vec![(10,8),(7,9),(11,5)])

    // Details: [dir:2]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ____________⊙______
    // ___________⊛_______
    // __________⊙________
    // _________⊕_________
    // _______⊖⊕⊕⊙________
    // _______⊙___________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((11,5), TypeOfThreat::ThreeOF, vec![(10,8),(10,6),(7,9),(12,4)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊕_________
    // _______⊖⊕⊕⊙________
    // _________⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,6), TypeOfThreat::ThreeOC, vec![(10,8),(9,9),(9,5)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊙_________
    // _________⊕_________
    // _______⊖⊕⊕⊙________
    // _________⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,5), TypeOfThreat::ThreeOF, vec![(10,8),(9,6),(9,9),(9,4)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊕_________
    // _______⊖⊕⊕⊙________
    // _________⊛_________
    // _________⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::ThreeOC, vec![(10,8),(9,6),(9,10)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊕_________
    // _______⊖⊕⊕⊙________
    // _________⊙_________
    // _________⊛_________
    // _________⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,10), TypeOfThreat::ThreeOF, vec![(10,8),(9,9),(9,6),(9,11)])
    #[test]
    fn threat_connect2_catchis() {
        let black_pos = vec![(9, 8), (9, 7), (8, 8)];
        let white_pos = vec![(7, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            (
                (7, 9),
                TypeOfThreat::ThreeOC,
                vec![(10, 8), (10, 6), (6, 10)],
            ),
            (
                (6, 10),
                TypeOfThreat::ThreeOF,
                vec![(10, 8), (7, 9), (10, 6), (5, 11)],
            ),
            (
                (10, 6),
                TypeOfThreat::ThreeOC,
                vec![(10, 8), (7, 9), (11, 5)],
            ),
            (
                (11, 5),
                TypeOfThreat::ThreeOF,
                vec![(10, 8), (10, 6), (7, 9), (12, 4)],
            ),
            ((9, 6), TypeOfThreat::ThreeOC, vec![(10, 8), (9, 9), (9, 5)]),
            (
                (9, 5),
                TypeOfThreat::ThreeOF,
                vec![(10, 8), (9, 6), (9, 9), (9, 4)],
            ),
            (
                (9, 9),
                TypeOfThreat::ThreeOC,
                vec![(10, 8), (9, 6), (9, 10)],
            ),
            (
                (9, 10),
                TypeOfThreat::ThreeOF,
                vec![(10, 8), (9, 9), (9, 6), (9, 11)],
            ),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _______⊖⊕__________
    // _________⊕_________
    // _______⊖⊕⊕_________
    // _______⊖⊕__________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:0]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ______⊙____________
    // _______⊛___________
    // _______⊖⊕_⊙________
    // _________⊕_________
    // _______⊖⊕⊕⊙________
    // _______⊖⊕__________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((7,5), TypeOfThreat::ThreeOC, vec![(10,6),(10,8),(6,4)])

    // Details: [dir:0]
    // ___________________
    // ___________________
    // ___________________
    // _____⊙_____________
    // ______⊛____________
    // _______⊙___________
    // _______⊖⊕_⊙________
    // _________⊕_________
    // _______⊖⊕⊕⊙________
    // _______⊖⊕__________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((6,4), TypeOfThreat::ThreeOF, vec![(10,6),(7,5),(10,8),(5,3)])

    // Details: [dir:0]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _______⊙___________
    // _______⊖⊕_⊙________
    // _________⊕_________
    // _______⊖⊕⊕⊛________
    // _______⊖⊕__⊙_______
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((10,8), TypeOfThreat::ThreeOC, vec![(10,6),(7,5),(11,9)])

    // Details: [dir:0]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _______⊙___________
    // _______⊖⊕_⊙________
    // _________⊕_________
    // _______⊖⊕⊕⊙________
    // _______⊖⊕__⊛_______
    // ____________⊙______
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((11,9), TypeOfThreat::ThreeOF, vec![(10,6),(10,8),(7,5),(12,10)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _______⊖⊕⊛⊙________
    // _________⊕_________
    // _______⊖⊕⊕⊙________
    // _______⊖⊕⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,6), TypeOfThreat::ThreeOC, vec![(10,6),(10,8),(9,9),(9,5)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _______⊖⊕⊙⊙________
    // _________⊕_________
    // _______⊖⊕⊕⊙________
    // _______⊖⊕⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,5), TypeOfThreat::ThreeOF, vec![(10,8),(10,6),(9,6),(9,9),(9,4)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _______⊖⊕⊙⊙________
    // _________⊕_________
    // _______⊖⊕⊕⊙________
    // _______⊖⊕⊛⊙________
    // _________⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::ThreeOC, vec![(10,9),(10,6),(10,8),(9,6),(9,10)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _______⊖⊕⊙⊙________
    // _________⊕_________
    // _______⊖⊕⊕⊙________
    // _______⊖⊕⊙_________
    // _________⊛_________
    // _________⊙⊙________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,10), TypeOfThreat::ThreeOF, vec![(10,11),(10,6),(10,8),(9,9),(9,6),(9,11)])
    #[test]
    fn threat_connect2_extremity1() {
        let black_pos = vec![(9, 8), (9, 7), (8, 9), (8, 6), (8, 8)];
        let white_pos = vec![(7, 9), (7, 6), (7, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            (
                (7, 5),
                TypeOfThreat::ThreeOC,
                vec![(10, 6), (10, 8), (6, 4)],
            ),
            (
                (6, 4),
                TypeOfThreat::ThreeOF,
                vec![(10, 6), (7, 5), (10, 8), (5, 3)],
            ),
            (
                (10, 8),
                TypeOfThreat::ThreeOC,
                vec![(10, 6), (7, 5), (11, 9)],
            ),
            (
                (11, 9),
                TypeOfThreat::ThreeOF,
                vec![(10, 6), (10, 8), (7, 5), (12, 10)],
            ),
            (
                (9, 6),
                TypeOfThreat::ThreeOC,
                vec![(10, 6), (10, 8), (9, 9), (9, 5)],
            ),
            (
                (9, 5),
                TypeOfThreat::ThreeOF,
                vec![(10, 8), (10, 6), (9, 6), (9, 9), (9, 4)],
            ),
            (
                (9, 9),
                TypeOfThreat::ThreeOC,
                vec![(10, 9), (10, 6), (10, 8), (9, 6), (9, 10)],
            ),
            (
                (9, 10),
                TypeOfThreat::ThreeOF,
                vec![(10, 11), (10, 6), (10, 8), (9, 9), (9, 6), (9, 11)],
            ),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
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
    // _______⊖⊕__________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖⊕⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,6), TypeOfThreat::ThreeOC, vec![(9,9),(9,5)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖⊕⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,5), TypeOfThreat::ThreeOF, vec![(9,6),(9,9),(9,4)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖⊕⊛⊙________
    // _________⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::ThreeOC, vec![(10,9),(9,6),(9,10)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖⊕⊙_________
    // _________⊛_________
    // _________⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,10), TypeOfThreat::ThreeOF, vec![(9,9),(9,6),(9,11)])
    #[test]
    fn threat_connect2_extremity2() {
        let black_pos = vec![(9, 8), (9, 7), (8, 9)];
        let white_pos = vec![(7, 9)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 6), TypeOfThreat::ThreeOC, vec![(9, 9), (9, 5)]),
            ((9, 5), TypeOfThreat::ThreeOF, vec![(9, 6), (9, 9), (9, 4)]),
            (
                (9, 9),
                TypeOfThreat::ThreeOC,
                vec![(10, 9), (9, 6), (9, 10)],
            ),
            (
                (9, 10),
                TypeOfThreat::ThreeOF,
                vec![(9, 9), (9, 6), (9, 11)],
            ),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
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
    // __________⊕⊖_______
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊕_________
    // _________⊕_________
    // _________⊙⊕⊖_______
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,6), TypeOfThreat::ThreeOC, vec![(9,9),(9,5)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // _________⊙⊕⊖_______
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,5), TypeOfThreat::ThreeOF, vec![(9,6),(9,9),(9,4)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // ________⊙⊛⊕⊖_______
    // _________⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::ThreeOC, vec![(8,9),(9,6),(9,10)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // _________⊙⊕⊖_______
    // _________⊛_________
    // _________⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,10), TypeOfThreat::ThreeOF, vec![(9,9),(9,6),(9,11)])
    #[test]
    fn threat_connect2_extremity3() {
        let black_pos = vec![(9, 8), (9, 7), (10, 9)];
        let white_pos = vec![(11, 9)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 6), TypeOfThreat::ThreeOC, vec![(9, 9), (9, 5)]),
            ((9, 5), TypeOfThreat::ThreeOF, vec![(9, 6), (9, 9), (9, 4)]),
            ((9, 9), TypeOfThreat::ThreeOC, vec![(8, 9), (9, 6), (9, 10)]),
            (
                (9, 10),
                TypeOfThreat::ThreeOF,
                vec![(9, 9), (9, 6), (9, 11)],
            ),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
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
    // ________⊕_⊖________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊕_________
    // _________⊕_________
    // ________⊕⊙⊖________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,6), TypeOfThreat::ThreeOC, vec![(9,9),(9,5)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // ________⊕⊙⊖________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,5), TypeOfThreat::ThreeOF, vec![(9,6),(9,9),(9,4)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // _______⊙⊕⊛⊖________
    // _________⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::ThreeOC, vec![(7,9),(9,6),(9,10)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // ________⊕⊙⊖________
    // _________⊛_________
    // _________⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,10), TypeOfThreat::ThreeOF, vec![(9,9),(9,6),(9,11)])
    #[test]
    fn threat_connect_2_catch_extremity_hard1_0() {
        let black_pos = vec![(9, 8), (9, 7), (8, 9)];
        let white_pos = vec![(10, 9)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 6), TypeOfThreat::ThreeOC, vec![(9, 9), (9, 5)]),
            ((9, 5), TypeOfThreat::ThreeOF, vec![(9, 6), (9, 9), (9, 4)]),
            ((9, 9), TypeOfThreat::ThreeOC, vec![(7, 9), (9, 6), (9, 10)]), // (7,9) missing currently
            (
                (9, 10),
                TypeOfThreat::ThreeOF,
                vec![(9, 9), (9, 6), (9, 11)],
            ),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
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
    // ________⊖_⊕________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊕_________
    // _________⊕_________
    // ________⊖⊙⊕________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,6), TypeOfThreat::ThreeOC, vec![(9,9),(9,5)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // ________⊖⊙⊕________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,5), TypeOfThreat::ThreeOF, vec![(9,6),(9,9),(9,4)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // ________⊖⊛⊕⊙_______
    // _________⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::ThreeOC, vec![(11,9),(9,6),(9,10)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // ________⊖⊙⊕________
    // _________⊛_________
    // _________⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,10), TypeOfThreat::ThreeOF, vec![(9,9),(9,6),(9,11)])
    #[test]
    fn threat_connect_2_catch_extremity_hard1_reverse() {
        let black_pos = vec![(9, 8), (9, 7), (10, 9)];
        let white_pos = vec![(8, 9)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 6), TypeOfThreat::ThreeOC, vec![(9, 9), (9, 5)]),
            ((9, 5), TypeOfThreat::ThreeOF, vec![(9, 6), (9, 9), (9, 4)]),
            (
                (9, 9),
                TypeOfThreat::ThreeOC,
                vec![(11, 9), (9, 6), (9, 10)],
            ),
            (
                (9, 10),
                TypeOfThreat::ThreeOF,
                vec![(9, 9), (9, 6), (9, 11)],
            ),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
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
    // _______⊖⊕__________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖⊕⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,6), TypeOfThreat::ThreeOC, vec![(9,9),(9,5)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖⊕⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,5), TypeOfThreat::ThreeOF, vec![(9,6),(9,9),(9,4)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖⊕⊛⊙________
    // _________⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::ThreeOC, vec![(10,9),(9,6),(9,10)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖⊕⊙_________
    // _________⊛_________
    // _________⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,10), TypeOfThreat::ThreeOF, vec![(9,9),(9,6),(9,11)])
    #[test]
    fn threat_connect_2_catch_extremity2() {
        let black_pos = vec![(9, 8), (9, 7), (8, 9)];
        let white_pos = vec![(7, 9)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 6), TypeOfThreat::ThreeOC, vec![(9, 9), (9, 5)]),
            ((9, 5), TypeOfThreat::ThreeOF, vec![(9, 6), (9, 9), (9, 4)]),
            (
                (9, 9),
                TypeOfThreat::ThreeOC,
                vec![(10, 9), (9, 6), (9, 10)],
            ),
            (
                (9, 10),
                TypeOfThreat::ThreeOF,
                vec![(9, 9), (9, 6), (9, 11)],
            ),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
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
    // _______⊖⊕_⊖________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖⊕⊙⊖________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,6), TypeOfThreat::ThreeOC, vec![(9,9),(9,5)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖⊕⊙⊖________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,5), TypeOfThreat::ThreeOF, vec![(9,6),(9,9),(9,4)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖⊕⊛⊖________
    // _________⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::ThreeOC, vec![(9,6),(9,10)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖⊕⊙⊖________
    // _________⊛_________
    // _________⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,10), TypeOfThreat::ThreeOF, vec![(9,9),(9,6),(9,11)])
    #[test]
    fn threat_connect_2_not_catch_extremity() {
        let black_pos = vec![(9, 8), (9, 7), (8, 9)];
        let white_pos = vec![(10, 9), (7, 9)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 6), TypeOfThreat::ThreeOC, vec![(9, 9), (9, 5)]),
            ((9, 5), TypeOfThreat::ThreeOF, vec![(9, 6), (9, 9), (9, 4)]),
            ((9, 9), TypeOfThreat::ThreeOC, vec![(9, 6), (9, 10)]),
            (
                (9, 10),
                TypeOfThreat::ThreeOF,
                vec![(9, 9), (9, 6), (9, 11)],
            ),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
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
    // _______⊖__⊖________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖_⊙⊖________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,6), TypeOfThreat::ThreeOC, vec![(9,9),(9,5)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖_⊙⊖________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,5), TypeOfThreat::ThreeOF, vec![(9,6),(9,9),(9,4)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _______⊖_⊛⊖________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::FiveTake, vec![])
    #[test]
    fn threat_connect_2_catch_9_in_a_row_first() {
        let black_pos = vec![(9, 8), (9, 7), (9, 10), (9, 11)];
        let white_pos = vec![(10, 9), (7, 9)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 6), TypeOfThreat::ThreeOC, vec![(9, 9), (9, 5)]),
            ((9, 5), TypeOfThreat::ThreeOF, vec![(9, 6), (9, 9), (9, 4)]),
            ((9, 9), TypeOfThreat::FiveTake, vec![]),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
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
    // ________⊕_⊖________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊕_________
    // _________⊕_________
    // ________⊕⊙⊖________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,6), TypeOfThreat::ThreeOC, vec![(9,9),(9,5)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // ________⊕⊙⊖________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,5), TypeOfThreat::ThreeOF, vec![(9,6),(9,9),(9,4)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _______⊙⊕⊛⊖________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::FiveTake, vec![(7,9)])
    #[test]
    fn threat_connect_2_catch_9_in_a_row_catch_0() {
        let black_pos = vec![(9, 8), (9, 7), (9, 10), (9, 11), (8, 9)];
        let white_pos = vec![(10, 9)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 6), TypeOfThreat::ThreeOC, vec![(9, 9), (9, 5)]),
            ((9, 5), TypeOfThreat::ThreeOF, vec![(9, 6), (9, 9), (9, 4)]),
            ((9, 9), TypeOfThreat::FiveTake, vec![(7, 9)]),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
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
    // ________⊕⊕⊖________
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

    // Details: [dir:2]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __________⊙________
    // _________⊕_________
    // _______⊙⊕⊕⊖________
    // _______⊛___________
    // ______⊙__⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((7,9), TypeOfThreat::ThreeOC, vec![(7,8),(10,6),(6,10)])

    // Details: [dir:2]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __________⊙________
    // _________⊕_________
    // _______⊙⊕⊕⊖________
    // _______⊙___________
    // ______⊛__⊕_________
    // _____⊙___⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((6,10), TypeOfThreat::ThreeOF, vec![(7,8),(7,9),(10,6),(5,11)])

    // Details: [dir:2]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________⊙_______
    // __________⊛________
    // _________⊕_________
    // _______⊙⊕⊕⊖________
    // _______⊙___________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((10,6), TypeOfThreat::ThreeOC, vec![(7,8),(7,9),(11,5)])

    // Details: [dir:2]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ____________⊙______
    // ___________⊛_______
    // __________⊙________
    // _________⊕_________
    // _______⊙⊕⊕⊖________
    // _______⊙___________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((11,5), TypeOfThreat::ThreeOF, vec![(7,8),(10,6),(7,9),(12,4)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊕_________
    // _______⊙⊕⊕⊖________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,6), TypeOfThreat::ThreeOC, vec![(7,8),(9,9),(9,5)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊙_________
    // _________⊕_________
    // _______⊙⊕⊕⊖________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,5), TypeOfThreat::ThreeOF, vec![(7,8),(9,6),(9,9),(9,4)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _______⊙⊕⊕⊖________
    // _________⊛_________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::FiveTake, vec![(7,8)])
    #[test]
    fn threat_connect_2_catch_9_in_a_row_catch_1() {
        let black_pos = vec![(9, 8), (9, 7), (9, 10), (9, 11), (8, 8)];
        let white_pos = vec![(10, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            (
                (7, 9),
                TypeOfThreat::ThreeOC,
                vec![(7, 8), (10, 6), (6, 10)],
            ),
            (
                (6, 10),
                TypeOfThreat::ThreeOF,
                vec![(7, 8), (7, 9), (10, 6), (5, 11)],
            ),
            (
                (10, 6),
                TypeOfThreat::ThreeOC,
                vec![(7, 8), (7, 9), (11, 5)],
            ),
            (
                (11, 5),
                TypeOfThreat::ThreeOF,
                vec![(7, 8), (10, 6), (7, 9), (12, 4)],
            ),
            ((9, 6), TypeOfThreat::ThreeOC, vec![(7, 8), (9, 9), (9, 5)]),
            (
                (9, 5),
                TypeOfThreat::ThreeOF,
                vec![(7, 8), (9, 6), (9, 9), (9, 4)],
            ),
            ((9, 9), TypeOfThreat::FiveTake, vec![(7, 8)]),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
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
    // ________⊕⊕⊖________
    // ___________________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊕_________
    // _______⊙⊕⊕⊖________
    // _________⊙_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,6), TypeOfThreat::ThreeOC, vec![(7,8),(9,9),(9,5)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊛_________
    // _________⊙_________
    // _________⊕_________
    // _______⊙⊕⊕⊖________
    // _________⊙_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,5), TypeOfThreat::ThreeOF, vec![(7,8),(9,6),(9,9),(9,4)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _______⊙⊕⊕⊖________
    // _________⊛_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::FourOC, vec![(7,8)])
    #[test]
    fn threat_connect_2_catch_9_in_a_row_catch_2() {
        let black_pos = vec![(9, 8), (9, 7), (9, 10), (8, 8)];
        let white_pos = vec![(10, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 6), TypeOfThreat::ThreeOC, vec![(7, 8), (9, 9), (9, 5)]),
            (
                (9, 5),
                TypeOfThreat::ThreeOF,
                vec![(7, 8), (9, 6), (9, 9), (9, 4)],
            ),
            ((9, 9), TypeOfThreat::FourOC, vec![(7, 8)]),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (9, 8),
            Some(false),
            expected_result
        ))
    }

    #[test]
    fn threat_connect_2_free_inside() {
        let black_pos = vec![(9, 8), (9, 10)];
        let white_pos = vec![(10, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 6), TypeOfThreat::ThreeOC, vec![(7, 8), (9, 9), (9, 5)]),
            // (
            //     (9, 5),
            //     TypeOfThreat::ThreeOF,
            //     vec![(7, 8), (9, 6), (9, 9), (9, 4)],
            // ),
            // ((9, 9), TypeOfThreat::FourOC, vec![(7, 8)]),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (9, 8),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊖_________
    // _________⊕_________
    // ________⊕⊕⊖________
    // ___________________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊖_________
    // _________⊕_________
    // _______⊙⊕⊕⊖________
    // _________⊛_________
    // _________⊕_________
    // _________⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::FourSOC, vec![(7,8),(9,11)])
    #[test]
    fn threat_connect_2_catch_9_in_a_row_catch_3() {
        let black_pos = vec![(9, 8), (9, 7), (9, 10), (8, 8)];
        let white_pos = vec![(10, 8), (9, 6)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((9, 9), TypeOfThreat::FourSOC, vec![(7, 8), (9, 11)])];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (9, 8),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊖_________
    // _________⊕_________
    // ________⊕⊕⊖________
    // ___________________
    // ___________________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊖_________
    // _________⊕_________
    // _______⊙⊕⊕⊖________
    // _________⊛_________
    // _________⊙_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::FourSOF, vec![(7,8),(9,10)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊖_________
    // _________⊕_________
    // _______⊙⊕⊕⊖________
    // _________⊙_________
    // _________⊛_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,10), TypeOfThreat::FourSOF, vec![(7,8),(9,9)])
    #[test]
    fn threat_connect_2_catch_9_in_a_row_catch_4() {
        let black_pos = vec![(9, 8), (9, 7), (9, 11), (8, 8)];
        let white_pos = vec![(10, 8), (9, 6)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 9), TypeOfThreat::FourSOF, vec![(7, 8), (9, 10)]),
            ((9, 10), TypeOfThreat::FourSOF, vec![(7, 8), (9, 9)]),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (9, 8),
            Some(false),
            expected_result
        ))
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
    // ________⊕⊕⊖________
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

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _______⊙⊕⊕⊖________
    // _________⊛_________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::FiveTake, vec![(7,8)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // ________⊕⊕⊖________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // _________⊛_________
    // _________⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,12), TypeOfThreat::ThreeOC, vec![(9,9),(9,13)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // ________⊕⊕⊖________
    // _________⊙_________
    // _________⊕_________
    // _________⊕_________
    // _________⊙_________
    // _________⊛_________
    // _________⊙_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,13), TypeOfThreat::ThreeOF, vec![(9,12),(9,9),(9,14)])
    #[test]
    fn threat_connect_2_catch_9_in_a_row_catch_5() {
        let black_pos = vec![(9, 8), (9, 7), (9, 10), (9, 11), (8, 8)];
        let white_pos = vec![(10, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 9), TypeOfThreat::FiveTake, vec![(7, 8)]),
            ((9, 12), TypeOfThreat::ThreeOC, vec![(9, 9), (9, 13)]),
            (
                (9, 13),
                TypeOfThreat::ThreeOF,
                vec![(9, 12), (9, 9), (9, 14)],
            ),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (9, 10),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:2]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ____⊙______________
    // ___⊛_______________
    // __⊕________________
    // ⊙⊕⊕⊖_______________
    // ⊙__________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((3,6), TypeOfThreat::ThreeOC, vec![(0,8),(0,9),(4,5)])

    // Details: [dir:2]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _____⊙_____________
    // ____⊛______________
    // ___⊙_______________
    // __⊕________________
    // ⊙⊕⊕⊖_______________
    // ⊙__________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,5), TypeOfThreat::ThreeOF, vec![(0,8),(3,6),(0,9),(5,4)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊙________________
    // __⊛________________
    // __⊕________________
    // ⊙⊕⊕⊖_______________
    // __⊙________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,6), TypeOfThreat::ThreeOC, vec![(0,8),(2,9),(2,5)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊙________________
    // __⊛________________
    // __⊙________________
    // __⊕________________
    // ⊙⊕⊕⊖_______________
    // __⊙________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,5), TypeOfThreat::ThreeOF, vec![(0,8),(2,6),(2,9),(2,4)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // __⊛________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,9), TypeOfThreat::FiveTake, vec![])
    #[test]
    fn threat_connect_2_fake_catch_9_in_a_row_other_position_0() {
        let black_pos = vec![(2, 8), (2, 7), (2, 10), (2, 11), (2, 12), (2, 13), (1, 8)];
        let white_pos = vec![(3, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((3, 6), TypeOfThreat::ThreeOC, vec![(0, 8), (0, 9), (4, 5)]),
            (
                (4, 5),
                TypeOfThreat::ThreeOF,
                vec![(0, 8), (3, 6), (0, 9), (5, 4)],
            ),
            ((2, 6), TypeOfThreat::ThreeOC, vec![(0, 8), (2, 9), (2, 5)]),
            (
                (2, 5),
                TypeOfThreat::ThreeOF,
                vec![(0, 8), (2, 6), (2, 9), (2, 4)],
            ),
            ((2, 9), TypeOfThreat::ThreeTake, vec![]),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (2, 7),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊙________________
    // __⊛________________
    // __⊕________________
    // ⊙⊕⊕⊖_______________
    // __⊙________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,6), TypeOfThreat::ThreeOC, vec![(0,8),(2,9),(2,5)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊙________________
    // __⊛________________
    // __⊙________________
    // __⊕________________
    // ⊙⊕⊕⊖_______________
    // __⊙________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,5), TypeOfThreat::ThreeOF, vec![(0,8),(2,6),(2,9),(2,4)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // __⊛________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,9), TypeOfThreat::ThreeTake, vec![])
    #[test]
    fn threat_connect_2_fake_catch_9_in_a_row_other_position_1() {
        let black_pos = vec![(2, 8), (2, 7), (2, 10), (2, 11), (2, 12), (2, 13), (1, 8)];
        let white_pos = vec![(3, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((2, 6), TypeOfThreat::ThreeOC, vec![(0, 8), (2, 9), (2, 5)]),
            (
                (2, 5),
                TypeOfThreat::ThreeOF,
                vec![(0, 8), (2, 6), (2, 9), (2, 4)],
            ),
            ((2, 9), TypeOfThreat::ThreeTake, vec![]),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (2, 8),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:2]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ____⊙______________
    // ___⊛_______________
    // __⊕________________
    // ⊙⊕⊕⊖_______________
    // ⊙__________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((3,6), TypeOfThreat::ThreeOC, vec![(0,8),(0,9),(4,5)])

    // Details: [dir:2]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _____⊙_____________
    // ____⊛______________
    // ___⊙_______________
    // __⊕________________
    // ⊙⊕⊕⊖_______________
    // ⊙__________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,5), TypeOfThreat::ThreeOF, vec![(0,8),(3,6),(0,9),(5,4)])
    #[test]
    fn threat_connect_2_fake_catch_9_in_a_row_other_position_2() {
        let black_pos = vec![(2, 8), (2, 7), (2, 10), (2, 11), (2, 12), (2, 13), (1, 8)];
        let white_pos = vec![(3, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((3, 6), TypeOfThreat::ThreeOC, vec![(0, 8), (0, 9), (4, 5)]),
            (
                (4, 5),
                TypeOfThreat::ThreeOF,
                vec![(0, 8), (3, 6), (0, 9), (5, 4)],
            ),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (1, 8),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ⊕__________________
    // _⊕⊖________________
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
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn threat_connect_2_diagonal_left_0() {
        let black_pos = vec![(0, 0), (1, 1)];
        let white_pos = vec![(2, 1)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (2, 2),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ⊕__________________
    // _⊕⊖________________
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
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn threat_connect_2_diagonal_left_1() {
        let black_pos = vec![(0, 0), (1, 1)];
        let white_pos = vec![(2, 1)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (1, 1),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ⊕__________________
    // _⊕⊖________________
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
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn threat_connect_2_diagonal_left_2() {
        let black_pos = vec![(0, 0), (1, 1)];
        let white_pos = vec![(2, 1)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (0, 0),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // _⊕_⊖_______________
    // __⊕________________
    // __⊕________________
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
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // _⊕⊙⊖_______________
    // __⊕________________
    // __⊕________________
    // __⊛________________
    // __⊙________________
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
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,3), TypeOfThreat::ThreeOC, vec![(2,0),(2,4)])

    // Details: [dir:3]
    // _⊕⊙⊖_______________
    // __⊕________________
    // __⊕________________
    // __⊙________________
    // __⊛________________
    // __⊙________________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,4), TypeOfThreat::ThreeOF, vec![(2,3),(2,0),(2,5)])
    #[test]
    fn threat_connect_2_fake_catch_close_top_2_s0_0() {
        let black_pos = vec![(2, 2), (2, 1), (1, 0)];
        let white_pos = vec![(3, 0)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((2, 3), TypeOfThreat::ThreeOC, vec![(2, 0), (2, 4)]),
            ((2, 4), TypeOfThreat::ThreeOF, vec![(2, 3), (2, 0), (2, 5)]),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (2, 2),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___⊕_______________
    // ___________________
    // ___⊕_______________
    // ___⊕_______________
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
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___⊕_______________
    // ___⊛_______________
    // ___⊕_______________
    // ___⊕_______________
    // ___⊙_______________
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
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((3,1), TypeOfThreat::FourSOC, vec![(3,4)])

    // Details: [dir:3]
    // ___⊕_______________
    // ___⊙_______________
    // ___⊕_______________
    // ___⊕_______________
    // ___⊛_______________
    // ___⊙_______________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((3,4), TypeOfThreat::ThreeOC, vec![(3,1),(3,5)])

    // Details: [dir:3]
    // ___⊕_______________
    // ___⊙_______________
    // ___⊕_______________
    // ___⊕_______________
    // ___⊙_______________
    // ___⊛_______________
    // ___⊙_______________
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
    // ___________________
    // DEFENSIVE_MOVE:
    // ((3,5), TypeOfThreat::ThreeOF, vec![(3,4),(3,1),(3,6)])
    #[test]
    fn threat_connect_2_fake_catch_close_top_2_s0_4() {
        let black_pos = vec![(3, 2), (3, 0), (3, 3)];
        let white_pos = vec![(3, 0)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((3, 1), TypeOfThreat::FourSOC, vec![(3, 4)]),
            ((3, 4), TypeOfThreat::ThreeOC, vec![(3, 1), (3, 5)]),
            ((3, 5), TypeOfThreat::ThreeOF, vec![(3, 4), (3, 1), (3, 6)]),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (3, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // _⊕_⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
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
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // _⊕⊙⊖_______________
    // __⊛________________
    // __⊕________________
    // __⊕________________
    // __⊙________________
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
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,1), TypeOfThreat::ThreeOC, vec![(2,4),(2,0)])

    // Details: [dir:3]
    // _⊕_⊖_______________
    // __⊙________________
    // __⊕________________
    // __⊕________________
    // __⊛________________
    // __⊙________________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,4), TypeOfThreat::ThreeOC, vec![(2,1),(2,5)])

    // Details: [dir:3]
    // _⊕_⊖_______________
    // __⊙________________
    // __⊕________________
    // __⊕________________
    // __⊙________________
    // __⊛________________
    // __⊙________________
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
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,5), TypeOfThreat::ThreeOF, vec![(2,4),(2,1),(2,6)])
    #[test]
    fn threat_connect_2_fake_catch_close_top_2_s0_1() {
        let black_pos = vec![(2, 2), (2, 3), (1, 0)];
        let white_pos = vec![(3, 0)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((2, 1), TypeOfThreat::ThreeOC, vec![(2, 4), (2, 0)]),
            ((2, 4), TypeOfThreat::ThreeOC, vec![(2, 1), (2, 5)]),
            ((2, 5), TypeOfThreat::ThreeOF, vec![(2, 4), (2, 1), (2, 6)]),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (2, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // _⊕_⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
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
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // _⊕⊙⊖_______________
    // __⊛________________
    // __⊕________________
    // __⊕________________
    // __⊙________________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,2), TypeOfThreat::ThreeOC, vec![(2,5),(2,1)])

    // Details: [dir:3]
    // __⊙________________
    // ⊙⊕⊛⊖_______________
    // __⊙________________
    // __⊕________________
    // __⊕________________
    // __⊙________________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,1), TypeOfThreat::ThreeOF, vec![(0,1),(2,2),(2,5),(2,0)])

    // Details: [dir:3]
    // ___________________
    // _⊕_⊖_______________
    // __⊙________________
    // __⊕________________
    // __⊕________________
    // __⊛________________
    // __⊙________________
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
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,5), TypeOfThreat::ThreeOC, vec![(2,2),(2,6)])

    // Details: [dir:3]
    // ___________________
    // _⊕_⊖_______________
    // __⊙________________
    // __⊕________________
    // __⊕________________
    // __⊙________________
    // __⊛________________
    // __⊙________________
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
    // DEFENSIVE_MOVE:
    // ((2,6), TypeOfThreat::ThreeOF, vec![(2,5),(2,2),(2,7)])
    #[test]
    fn threat_connect_2_fake_catch_close_top_2_s0_2() {
        let black_pos = vec![(2, 4), (2, 3), (1, 1)];
        let white_pos = vec![(3, 1)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((2, 2), TypeOfThreat::ThreeOC, vec![(2, 5), (2, 1)]),
            (
                (2, 1),
                TypeOfThreat::ThreeOF,
                vec![(0, 1), (2, 2), (2, 5), (2, 0)],
            ),
            ((2, 5), TypeOfThreat::ThreeOC, vec![(2, 2), (2, 6)]),
            ((2, 6), TypeOfThreat::ThreeOF, vec![(2, 5), (2, 2), (2, 7)]),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (2, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // _⊕_⊖_______________
    // __⊕________________
    // __⊕________________
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
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // _⊕⊙⊖_______________
    // __⊕________________
    // __⊕________________
    // __⊛________________
    // __⊙________________
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
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,3), TypeOfThreat::ThreeOC, vec![(2,0),(2,4)])

    // Details: [dir:3]
    // _⊕⊙⊖_______________
    // __⊕________________
    // __⊕________________
    // __⊙________________
    // __⊛________________
    // __⊙________________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,4), TypeOfThreat::ThreeOF, vec![(2,3),(2,0),(2,5)])
    #[test]
    fn threat_connect_2_fake_catch_close_top_2_s0_3() {
        let black_pos = vec![(2, 2), (2, 1), (1, 0)];
        let white_pos = vec![(3, 0)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((2, 3), TypeOfThreat::ThreeOC, vec![(2, 0), (2, 4)]),
            ((2, 4), TypeOfThreat::ThreeOF, vec![(2, 3), (2, 0), (2, 5)]),
        ];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (2, 1),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // ___________________
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
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊙_________
    // _________⊕_________
    // _________⊛_________
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
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,6), TypeOfThreat::FourSOC, vec![(9,4)])
    #[test]
    fn threat_connect_2_space_1_align_1_blocked() {
        let black_pos = vec![(9, 5), (9, 7), (9, 8)];
        let white_pos = vec![(9, 9)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((9, 6), TypeOfThreat::FourSOC, vec![(9, 4)])];
        assert!(test_threat_2(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
    }

    fn test_threat_3(
        white_pos: Vec<(usize, usize)>,
        black_pos: Vec<(usize, usize)>,
        pos2check: (usize, usize),
        actual_player: Option<bool>,
        expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)>,
    ) -> bool {
        let mut test_bboard = [[None; SIZE_BOARD]; SIZE_BOARD];
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
//        for i in 0..19 {
//            print!("// ");
//            for j in 0..19 {
//                match test_board[j][i] {
//                    Some(true) => print!("⊖"),
//                    Some(false) => print!("⊕"),
//                    None => print!("_"),
//                }
//            }
//            println!();
//        }
        let mut score_board = heuristic::evaluate_board(&mut test_board);
        let mut record: [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD] =
            initialize_record(&mut test_board, actual_player);
        // let mut threat_board: Vec<Vec<Vec<(TypeOfThreat, Vec<(usize,usize)>)>>> = (0..SIZE_BOARD).map(|_|
        //         (0..SIZE_BOARD).map(|_|
        //             Vec::with_capacity(AVRG_MAX_MULTIPLE_THREATS)
        //         ).collect()
        //  ).collect();

        // let mut tmp_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![];
        let mut global_results: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![];
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
            let tmp_result = match score_board[pos2check.0][pos2check.1][dir].0 {
                // 4 => { connect_4(pos2check, &mut score_board, &mut test_board, &mut record, actual_player, actual_take, dir) },
                // 3 => { connect_4(pos2check, &mut score_board, &mut test_board, &mut record, actual_player, actual_take, dir) },
                // 2 => { println!("OUPS_I_DID_IT_AGAIN") ; connect_2(&mut test_board, &mut score_board, &mut record, actual_player, pos2check, dir) }
                3 => connect_3(
                    &mut test_board,
                    &mut score_board,
                    &mut record,
                    actual_player,
                    pos2check,
                    dir,
                ),
                _ => vec![],
            };
            if tmp_result.len() == 0 {
                continue;
            }
            // tmp_result = connect_2(&mut test_board, &mut score_board, &mut record, actual_player, pos2check, 3);
            // println!("DEBUT°°°DEBUG_CONNECT: len({})", tmp_result.len());
            // tmp_result.iter().for_each(|((x,y), typeOfThreat, Opp)| {  threat_board[*x][*y].push((*typeOfThreat, Opp.clone())) } );
            // ret_debug.iter().for_each(|&x| print!("{}", x));
            // println!("DEBUG_CONNECT: len({})", tmp_result.len());
            tmp_result
                .iter()
                .for_each(|(defensive_move, type_of_threat, opp)| {
                    global_results.push((*defensive_move, *type_of_threat, (*opp).clone()));
                    // println!("-----------------");
                    // For each result, print the details of the threat + possible response
                    println!("\n// Details: [dir:{}]", dir);
                    for i in 0..19 {
                        print!("// ");
                        for j in 0..19 {
                            // Print specific attack move
                            if (defensive_move.0, defensive_move.1) == (j as usize, i as usize) {
                                print!("⊛")
                            } else if opp.contains(&(j, i)) {
                                print!("⊙")
                            } else {
                                match test_board.get_pawn(j, i) {
                                    Some(true) => print!("⊖"),
                                    Some(false) => print!("⊕"),
                                    None => print!("_"),
                                }
                            }
                        }
                        println!();
                    }
                    // ((9,4), TypeOfThreat::FourOF, vec![(10,8)]),
                    println!("// DEFENSIVE_MOVE:");

                    print!("// (({},{}), ", defensive_move.0, defensive_move.1);
                    print!(
                        "TypeOfThreat::{}, vec![",
                        match type_of_threat {
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
                    opp.iter().enumerate().for_each(|(i, (x, y))| {
                        if i == (opp.len() - 1) {
                            print!("({},{})", x, y)
                        } else {
                            print!("({},{}),", x, y)
                        }
                    });
                    println!("])");
                });
        }

        // print expected datastruct in test
        println!();
        global_results
            .iter()
            .enumerate()
            .for_each(|(j, (defensive_move, type_of_threat, opp))| {
                print!("(({},{}), ", defensive_move.0, defensive_move.1);
                print!(
                    "TypeOfThreat::{}, vec![",
                    match type_of_threat {
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
                opp.iter().enumerate().for_each(|(i, (x, y))| {
                    if i == (opp.len() - 1) {
                        print!("({},{})", x, y)
                    } else {
                        print!("({},{}),", x, y)
                    }
                });
                if j == (global_results.len() - 1) {
                    println!("])");
                } else {
                    println!("]),");
                }
            });
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
        //                 TypeOfThreat::FiveTake => println!("FiveTake"),
        //                 TypeOfThreat::FourOF => println!("FourOF"),
        //                 TypeOfThreat::FourSOF => println!("FourSOF"),
        //                 TypeOfThreat::TAKE => println!("TAKE"),
        //                 TypeOfThreat::ThreeOF => println!("ThreeOF"),
        //             }
        //             println!("Responses:");
        //             opp.iter().for_each(|(x,y)| println!("({},{})", x, y));
        //         });
        //     });
        // });
        global_results == expected_result
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊖______________
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

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊖______________
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
    // DEFENSIVE_MOVE:
    // ((4,2), TypeOfThreat::FourOC, vec![])

    // Details: [dir:3]
    // ___________________
    // ____⊛______________
    // ____⊙______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊖______________
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
    // DEFENSIVE_MOVE:
    // ((4,1), TypeOfThreat::FourSOF, vec![(4,2)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊙______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊖______________
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
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::FourSOC, vec![(4,2)])
    // _000_#
    #[test]
    fn threat_connect_3_space_1_blocked() {
        let black_pos = vec![(4, 3), (4, 4), (4, 5)];
        let white_pos = vec![(4, 7)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 2), TypeOfThreat::FourOC, vec![]),
            ((4, 1), TypeOfThreat::FourSOF, vec![(4, 2)]),
            ((4, 6), TypeOfThreat::FourSOC, vec![(4, 2)]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
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
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊙______________
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
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,0), TypeOfThreat::FourSOC, vec![(4,4)])

    // Details: [dir:3]
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
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
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,4), TypeOfThreat::FourOC, vec![])

    // Details: [dir:3]
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊙______________
    // ____⊛______________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,5), TypeOfThreat::FourSOF, vec![(4,4)])
    // _000_|
    #[test]
    fn threat_connect_3_space_1_edged() {
        let black_pos = vec![(4, 1), (4, 2), (4, 3)];
        let white_pos = vec![];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 0), TypeOfThreat::FourSOC, vec![(4, 4)]),
            ((4, 4), TypeOfThreat::FourOC, vec![]),
            ((4, 5), TypeOfThreat::FourSOF, vec![(4, 4)]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
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

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
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
    // DEFENSIVE_MOVE:
    // ((4,2), TypeOfThreat::FourOC, vec![])

    // Details: [dir:3]
    // ___________________
    // ____⊛______________
    // ____⊙______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
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
    // DEFENSIVE_MOVE:
    // ((4,1), TypeOfThreat::FourSOF, vec![(4,2)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
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
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::FiveTake, vec![])
    // _000_0_
    #[test]
    fn threat_connect_3_space_1_align_1_free() {
        let black_pos = vec![(4, 3), (4, 4), (4, 5), (4, 7)];
        let white_pos = vec![];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 2), TypeOfThreat::FourOC, vec![]),
            ((4, 1), TypeOfThreat::FourSOF, vec![(4, 2)]),
            ((4, 6), TypeOfThreat::FiveTake, vec![]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊖______________
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

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊖______________
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
    // DEFENSIVE_MOVE:
    // ((4,2), TypeOfThreat::FourOC, vec![])

    // Details: [dir:3]
    // ___________________
    // ____⊛______________
    // ____⊙______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊖______________
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
    // DEFENSIVE_MOVE:
    // ((4,1), TypeOfThreat::FourSOF, vec![(4,2)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
    // ____⊖______________
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
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::FiveTake, vec![])
    // _000_0#
    #[test]
    fn threat_connect_3_space_1_align_1_blocked() {
        let black_pos = vec![(4, 3), (4, 4), (4, 5), (4, 7)];
        let white_pos = vec![(4, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 2), TypeOfThreat::FourOC, vec![]),
            ((4, 1), TypeOfThreat::FourSOF, vec![(4, 2)]),
            ((4, 6), TypeOfThreat::FiveTake, vec![]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
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
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
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
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,1), TypeOfThreat::FiveTake, vec![])

    // Details: [dir:3]
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,5), TypeOfThreat::FourOC, vec![])

    // Details: [dir:3]
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊙______________
    // ____⊛______________
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
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::FourSOF, vec![(4,5)])
    // _000_0|
    #[test]
    fn threat_connect_3_space_1_align_1_edged() {
        let black_pos = vec![(4, 0), (4, 2), (4, 3), (4, 4)];
        let white_pos = vec![];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 1), TypeOfThreat::FiveTake, vec![]),
            ((4, 5), TypeOfThreat::FourOC, vec![]),
            ((4, 6), TypeOfThreat::FourSOF, vec![(4, 5)]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
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

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
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
    // DEFENSIVE_MOVE:
    // ((4,2), TypeOfThreat::FourOC, vec![])

    // Details: [dir:3]
    // ___________________
    // ____⊛______________
    // ____⊙______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
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
    // DEFENSIVE_MOVE:
    // ((4,1), TypeOfThreat::FourSOF, vec![(4,2)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
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
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::FourTake, vec![])
    // _000_00_
    #[test]
    fn threat_connect_3_space_1_align_2_free() {
        let black_pos = vec![(4, 3), (4, 4), (4, 5), (4, 7), (4, 8)];
        let white_pos = vec![];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 2), TypeOfThreat::FourOC, vec![]),
            ((4, 1), TypeOfThreat::FourSOF, vec![(4, 2)]),
            ((4, 6), TypeOfThreat::FourTake, vec![]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,2), TypeOfThreat::FourOC, vec![])

    // Details: [dir:3]
    // ___________________
    // ____⊛______________
    // ____⊙______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,1), TypeOfThreat::FourSOF, vec![(4,2)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::FourTake, vec![])
    // _000_00#
    #[test]
    fn threat_connect_3_space_1_align_2_blocked_simple() {
        let black_pos = vec![(4, 3), (4, 4), (4, 5), (4, 7), (4, 8)];
        let white_pos = vec![(4, 9)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 2), TypeOfThreat::FourOC, vec![]),
            ((4, 1), TypeOfThreat::FourSOF, vec![(4, 2)]),
            ((4, 6), TypeOfThreat::FourTake, vec![]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
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
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,2), TypeOfThreat::FourTake, vec![])

    // Details: [dir:3]
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
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
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::FourOC, vec![])

    // Details: [dir:3]
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊙______________
    // ____⊛______________
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
    // DEFENSIVE_MOVE:
    // ((4,7), TypeOfThreat::FourSOF, vec![(4,6)])
    // _000_00|
    #[test]
    fn threat_connect_3_space_1_align_2_edged() {
        let black_pos = vec![(4, 0), (4, 1), (4, 3), (4, 4), (4, 5)];
        let white_pos = vec![];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 2), TypeOfThreat::FourTake, vec![]),
            ((4, 6), TypeOfThreat::FourOC, vec![]),
            ((4, 7), TypeOfThreat::FourSOF, vec![(4, 6)]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,2), TypeOfThreat::FourOC, vec![])

    // Details: [dir:3]
    // ___________________
    // ____⊛______________
    // ____⊙______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,1), TypeOfThreat::FourSOF, vec![(4,2)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::ThreeTake, vec![])
    // _000_000_
    #[test]
    fn threat_connect_3_space_1_align_3_free() {
        let black_pos = vec![(4, 3), (4, 4), (4, 5), (4, 7), (4, 8), (4, 9)];
        let white_pos = vec![];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 2), TypeOfThreat::FourOC, vec![]),
            ((4, 1), TypeOfThreat::FourSOF, vec![(4, 2)]),
            ((4, 6), TypeOfThreat::ThreeTake, vec![]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,2), TypeOfThreat::FourOC, vec![])

    // Details: [dir:3]
    // ___________________
    // ____⊛______________
    // ____⊙______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,1), TypeOfThreat::FourSOF, vec![(4,2)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::ThreeTake, vec![])
    // _000_000#
    #[test]
    fn threat_connect_3_space_1_align_3_blocked_simple() {
        let black_pos = vec![(4, 3), (4, 4), (4, 5), (4, 7), (4, 8), (4, 9)];
        let white_pos = vec![(4, 10)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 2), TypeOfThreat::FourOC, vec![]),
            ((4, 1), TypeOfThreat::FourSOF, vec![(4, 2)]),
            ((4, 6), TypeOfThreat::ThreeTake, vec![]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕⊕⊖____________
    // ____⊕⊕⊖____________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___⊙⊛______________
    // ____⊕______________
    // ___⊙⊕⊕⊖____________
    // ___⊙⊕⊕⊖____________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,2), TypeOfThreat::FourOC, vec![(3,5),(3,4),(3,2)])

    // Details: [dir:3]
    // ___________________
    // ____⊛______________
    // ___⊙⊙______________
    // ____⊕______________
    // ___⊙⊕⊕⊖____________
    // ___⊙⊕⊕⊖____________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,1), TypeOfThreat::FourSOF, vec![(3,5),(3,4),(3,2),(4,2)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕⊕⊖____________
    // ___⊙⊕⊕⊖____________
    // ____⊛______________
    // ___⊙⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::ThreeTake, vec![(3,7),(3,5)])
    // _000_000#
    #[test]
    fn threat_connect_3_space_1_align_3_blocked_simple_with_takes() {
        let black_pos = vec![
            (4, 3),
            (4, 4),
            (4, 5),
            (4, 7),
            (4, 8),
            (4, 9),
            (5, 4),
            (5, 5),
        ];
        let white_pos = vec![(4, 10), (6, 4), (6, 5)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 2), TypeOfThreat::FourOC, vec![(3, 5), (3, 4), (3, 2)]),
            (
                (4, 1),
                TypeOfThreat::FourSOF,
                vec![(3, 5), (3, 4), (3, 2), (4, 2)],
            ),
            ((4, 6), TypeOfThreat::ThreeTake, vec![(3, 7), (3, 5)]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
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
    // ___________________

    // Details: [dir:3]
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
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
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,3), TypeOfThreat::ThreeTake, vec![])

    // Details: [dir:3]
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
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
    // DEFENSIVE_MOVE:
    // ((4,7), TypeOfThreat::FourOC, vec![])

    // Details: [dir:3]
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊙______________
    // ____⊛______________
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
    // DEFENSIVE_MOVE:
    // ((4,8), TypeOfThreat::FourSOF, vec![(4,7)])
    // _000_000|
    #[test]
    fn threat_connect_3_space_1_align_3_edged() {
        let black_pos = vec![(4, 0), (4, 1), (4, 2), (4, 4), (4, 5), (4, 6)];
        let white_pos = vec![];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 3), TypeOfThreat::ThreeTake, vec![]),
            ((4, 7), TypeOfThreat::FourOC, vec![]),
            ((4, 8), TypeOfThreat::FourSOF, vec![(4, 7)]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 4),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,2), TypeOfThreat::FourOC, vec![])

    // Details: [dir:3]
    // ___________________
    // ____⊛______________
    // ____⊙______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,1), TypeOfThreat::FourSOF, vec![(4,2)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::TwoTake, vec![])
    // _000_0000_
    #[test]
    fn threat_connect_3_space_1_align_4_free() {
        let black_pos = vec![(4, 3), (4, 4), (4, 5), (4, 7), (4, 8), (4, 9), (4, 10)];
        let white_pos = vec![];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 2), TypeOfThreat::FourOC, vec![]),
            ((4, 1), TypeOfThreat::FourSOF, vec![(4, 2)]),
            ((4, 6), TypeOfThreat::TwoTake, vec![]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,2), TypeOfThreat::FourOC, vec![])

    // Details: [dir:3]
    // ___________________
    // ____⊛______________
    // ____⊙______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,1), TypeOfThreat::FourSOF, vec![(4,2)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::TwoTake, vec![])
    // _000_0000#
    #[test]
    fn threat_connect_3_space_1_align_4_blocked_simple() {
        let black_pos = vec![(4, 3), (4, 4), (4, 5), (4, 7), (4, 8), (4, 9), (4, 10)];
        let white_pos = vec![(4, 11)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 2), TypeOfThreat::FourOC, vec![]),
            ((4, 1), TypeOfThreat::FourSOF, vec![(4, 2)]),
            ((4, 6), TypeOfThreat::TwoTake, vec![]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
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

    // Details: [dir:3]
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
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
    // DEFENSIVE_MOVE:
    // ((4,4), TypeOfThreat::TwoTake, vec![])

    // Details: [dir:3]
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
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
    // DEFENSIVE_MOVE:
    // ((4,8), TypeOfThreat::FourOC, vec![])

    // Details: [dir:3]
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊙______________
    // ____⊛______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,9), TypeOfThreat::FourSOF, vec![(4,8)])
    #[test]
    fn threat_connect_3_space_1_align_4_edged() {
        let black_pos = vec![(4, 0), (4, 1), (4, 2), (4, 3), (4, 5), (4, 6), (4, 7)];
        let white_pos = vec![];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 4), TypeOfThreat::TwoTake, vec![]),
            ((4, 8), TypeOfThreat::FourOC, vec![]),
            ((4, 9), TypeOfThreat::FourSOF, vec![(4, 8)]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 5),
            Some(false),
            expected_result
        ))
    }

    // _000___
    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
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
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,2), TypeOfThreat::FourOC, vec![])

    // Details: [dir:3]
    // ___________________
    // ____⊛______________
    // ____⊙______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,1), TypeOfThreat::FourSOF, vec![(4,2)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
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
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::FourOC, vec![])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊙______________
    // ____⊛______________
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
    // DEFENSIVE_MOVE:
    // ((4,7), TypeOfThreat::FourSOF, vec![(4,6)])
    #[test]
    fn threat_connect_3_space_2_free() {
        let black_pos = vec![(4, 3), (4, 4), (4, 5)];
        let white_pos = vec![];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 2), TypeOfThreat::FourOC, vec![]),
            ((4, 1), TypeOfThreat::FourSOF, vec![(4, 2)]),
            ((4, 6), TypeOfThreat::FourOC, vec![]),
            ((4, 7), TypeOfThreat::FourSOF, vec![(4, 6)]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ___________________
    // ____⊖______________
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

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ___________________
    // ____⊖______________
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
    // DEFENSIVE_MOVE:
    // ((4,2), TypeOfThreat::FourOC, vec![])

    // Details: [dir:3]
    // ___________________
    // ____⊛______________
    // ____⊙______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ___________________
    // ____⊖______________
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
    // DEFENSIVE_MOVE:
    // ((4,1), TypeOfThreat::FourSOF, vec![(4,2)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ___________________
    // ____⊖______________
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
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::FourOC, vec![])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊙______________
    // ____⊛______________
    // ____⊖______________
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
    // DEFENSIVE_MOVE:
    // ((4,7), TypeOfThreat::FourSOF, vec![(4,6)])
    // _000__#
    #[test]
    fn threat_connect_3_space_2_blocked_simple() {
        let black_pos = vec![(4, 3), (4, 4), (4, 5)];
        let white_pos = vec![(4, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 2), TypeOfThreat::FourOC, vec![]),
            ((4, 1), TypeOfThreat::FourSOF, vec![(4, 2)]),
            ((4, 6), TypeOfThreat::FourOC, vec![]),
            ((4, 7), TypeOfThreat::FourSOF, vec![(4, 6)]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
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
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
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
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,1), TypeOfThreat::FourOC, vec![])

    // Details: [dir:3]
    // ____⊛______________
    // ____⊙______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
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
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,0), TypeOfThreat::FourSOF, vec![(4,1)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,5), TypeOfThreat::FourOC, vec![])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊙______________
    // ____⊛______________
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
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::FourSOF, vec![(4,5)])
    // _000__|
    #[test]
    fn threat_connect_3_space_2_edged() {
        let black_pos = vec![(4, 2), (4, 3), (4, 4)];
        let white_pos = vec![];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 1), TypeOfThreat::FourOC, vec![]),
            ((4, 0), TypeOfThreat::FourSOF, vec![(4, 1)]),
            ((4, 5), TypeOfThreat::FourOC, vec![]),
            ((4, 6), TypeOfThreat::FourSOF, vec![(4, 5)]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ___________________
    // ____⊕______________
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

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ___________________
    // ____⊕______________
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
    // DEFENSIVE_MOVE:
    // ((4,2), TypeOfThreat::FourOC, vec![])

    // Details: [dir:3]
    // ___________________
    // ____⊛______________
    // ____⊙______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ___________________
    // ____⊕______________
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
    // DEFENSIVE_MOVE:
    // ((4,1), TypeOfThreat::FourSOF, vec![(4,2)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ___________________
    // ____⊕______________
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
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::FourOC, vec![])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊙______________
    // ____⊛______________
    // ____⊕______________
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
    // DEFENSIVE_MOVE:
    // ((4,7), TypeOfThreat::FourSOF, vec![(4,6)])
    // _000__0_
    #[test]
    fn threat_connect_3_space_2_align_1() {
        let black_pos = vec![(4, 3), (4, 4), (4, 5), (4, 8)];
        let white_pos = vec![];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 2), TypeOfThreat::FourOC, vec![]),
            ((4, 1), TypeOfThreat::FourSOF, vec![(4, 2)]),
            ((4, 6), TypeOfThreat::FourOC, vec![]),
            ((4, 7), TypeOfThreat::FourSOF, vec![(4, 6)]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
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

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
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
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::FiveTake, vec![])
    // #000_0_
    #[test]
    fn threat_connect_3_space_1_align_1_blocked_free() {
        let black_pos = vec![(4, 3), (4, 4), (4, 5), (4, 7)];
        let white_pos = vec![(4, 2)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((4, 6), TypeOfThreat::FiveTake, vec![])];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊖______________
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

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
    // ____⊖______________
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
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::FiveTake, vec![])
    // #000_0#
    #[test]
    fn threat_connect_3_space_1_align_1_blocked_blocked() {
        let black_pos = vec![(4, 3), (4, 4), (4, 5), (4, 7)];
        let white_pos = vec![(4, 2), (4, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((4, 6), TypeOfThreat::FiveTake, vec![])];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
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
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,1), TypeOfThreat::FiveTake, vec![])
    // #000_0|
    #[test]
    fn threat_connect_3_space_1_align_1_blocked_edged() {
        let black_pos = vec![(4, 0), (4, 2), (4, 3), (4, 4)];
        let white_pos = vec![(4, 5)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((4, 1), TypeOfThreat::FiveTake, vec![])];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
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

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
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
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::FourTake, vec![])
    // #000_00_
    #[test]
    fn threat_connect_3_space_1_align_2_blocked_free() {
        let black_pos = vec![(4, 3), (4, 4), (4, 5), (4, 7), (4, 8)];
        let white_pos = vec![(4, 2)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((4, 6), TypeOfThreat::FourTake, vec![])];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::FourTake, vec![])
    // #000_00#
    #[test]
    fn threat_connect_3_space_1_align_2_blocked_blocked() {
        let black_pos = vec![(4, 3), (4, 4), (4, 5), (4, 7), (4, 8)];
        let white_pos = vec![(4, 2), (4, 9)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((4, 6), TypeOfThreat::FourTake, vec![])];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
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
    // ___________________

    // Details: [dir:3]
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
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
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,2), TypeOfThreat::FourTake, vec![])
    // #000_00|
    #[test]
    fn threat_connect_3_space_1_align_2_blocked_edged() {
        let black_pos = vec![(4, 0), (4, 1), (4, 3), (4, 4), (4, 5)];
        let white_pos = vec![(4, 6)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((4, 2), TypeOfThreat::FourTake, vec![])];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::ThreeTake, vec![])
    // #000_000_
    #[test]
    fn threat_connect_3_space_1_align_3_blocked_free() {
        let black_pos = vec![(4, 3), (4, 4), (4, 5), (4, 7), (4, 8), (4, 9)];
        let white_pos = vec![(4, 2)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((4, 6), TypeOfThreat::ThreeTake, vec![])];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::ThreeTake, vec![])
    // #000_000#
    #[test]
    fn threat_connect_3_space_1_align_3_blocked_blocked() {
        let black_pos = vec![(4, 3), (4, 4), (4, 5), (4, 7), (4, 8), (4, 9)];
        let white_pos = vec![(4, 2), (4, 10)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((4, 6), TypeOfThreat::ThreeTake, vec![])];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
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

    // Details: [dir:3]
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
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
    // DEFENSIVE_MOVE:
    // ((4,3), TypeOfThreat::ThreeTake, vec![])
    // #000_000|
    #[test]
    fn threat_connect_3_space_1_align_3_blocked_edged() {
        let black_pos = vec![(4, 0), (4, 1), (4, 2), (4, 4), (4, 5), (4, 6)];
        let white_pos = vec![(4, 7)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((4, 3), TypeOfThreat::ThreeTake, vec![])];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 4),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::TwoTake, vec![])
    // #000_0000_
    #[test]
    fn threat_connect_3_space_1_align_4_blocked_free() {
        let black_pos = vec![(4, 3), (4, 4), (4, 5), (4, 7), (4, 8), (4, 9), (4, 10)];
        let white_pos = vec![(4, 2)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((4, 6), TypeOfThreat::TwoTake, vec![])];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::TwoTake, vec![])
    // #000_0000#
    #[test]
    fn threat_connect_3_space_1_align_4_blocked_blocked() {
        let black_pos = vec![(4, 3), (4, 4), (4, 5), (4, 7), (4, 8), (4, 9), (4, 10)];
        let white_pos = vec![(4, 2), (4, 11)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((4, 6), TypeOfThreat::TwoTake, vec![])];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
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

    // Details: [dir:3]
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
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
    // DEFENSIVE_MOVE:
    // ((4,4), TypeOfThreat::TwoTake, vec![])
    // #000_0000|
    #[test]
    fn threat_connect_3_space_1_align_4_blocked_edged() {
        let black_pos = vec![(4, 0), (4, 1), (4, 2), (4, 3), (4, 5), (4, 6), (4, 7)];
        let white_pos = vec![(4, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((4, 4), TypeOfThreat::TwoTake, vec![])];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 5),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
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
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊙______________
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
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::FourSOC, vec![(4,7)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊙______________
    // ____⊛______________
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
    // DEFENSIVE_MOVE:
    // ((4,7), TypeOfThreat::FourSOF, vec![(4,6)])
    // #000___
    #[test]
    fn threat_connect_3_space_2_blocked_free() {
        let black_pos = vec![(4, 3), (4, 4), (4, 5)];
        let white_pos = vec![(4, 2)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 6), TypeOfThreat::FourSOC, vec![(4, 7)]),
            ((4, 7), TypeOfThreat::FourSOF, vec![(4, 6)]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ___________________
    // ____⊖______________
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

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊙______________
    // ____⊖______________
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
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::FourSOC, vec![(4,7)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊙______________
    // ____⊛______________
    // ____⊖______________
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
    // DEFENSIVE_MOVE:
    // ((4,7), TypeOfThreat::FourSOF, vec![(4,6)])
    // #000__#
    #[test]
    fn threat_connect_3_space_2_blocked_blocked() {
        let black_pos = vec![(4, 3), (4, 4), (4, 5)];
        let white_pos = vec![(4, 2), (4, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 6), TypeOfThreat::FourSOC, vec![(4, 7)]),
            ((4, 7), TypeOfThreat::FourSOF, vec![(4, 6)]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
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
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ____⊙______________
    // ____⊛______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,1), TypeOfThreat::FourSOC, vec![(4,0)])

    // Details: [dir:3]
    // ____⊛______________
    // ____⊙______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊖______________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,0), TypeOfThreat::FourSOF, vec![(4,1)])
    // #000__|
    #[test]
    fn threat_connect_3_space_2_blocked_edged() {
        let black_pos = vec![(4, 2), (4, 3), (4, 4)];
        let white_pos = vec![(4, 5)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 1), TypeOfThreat::FourSOC, vec![(4, 0)]),
            ((4, 0), TypeOfThreat::FourSOF, vec![(4, 1)]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ___________________
    // ___________________
    // ____⊕______________
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

    // Details: [dir:3]
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊛______________
    // ____⊙______________
    // ____⊕______________
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
    // DEFENSIVE_MOVE:
    // ((4,5), TypeOfThreat::FourSOC, vec![(4,6)])

    // Details: [dir:3]
    // ___________________
    // ____⊖______________
    // ____⊕______________
    // ____⊕______________
    // ____⊕______________
    // ____⊙______________
    // ____⊛______________
    // ____⊕______________
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
    // DEFENSIVE_MOVE:
    // ((4,6), TypeOfThreat::FourSOF, vec![(4,5)])
    // #000__0?
    #[test]
    fn threat_connect_3_space_2_blocked_align_1() {
        let black_pos = vec![(4, 2), (4, 3), (4, 4), (4, 7)];
        let white_pos = vec![(4, 1)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 5), TypeOfThreat::FourSOC, vec![(4, 6)]),
            ((4, 6), TypeOfThreat::FourSOF, vec![(4, 5)]),
        ];
        assert!(test_threat_3(
            white_pos,
            black_pos,
            (4, 3),
            Some(false),
            expected_result
        ))
    }

    fn test_threat_4(
        white_pos: Vec<(usize, usize)>,
        black_pos: Vec<(usize, usize)>,
        pos2check: (usize, usize),
        actual_player: Option<bool>,
        expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)>,
    ) -> bool {
        let mut test_bboard = [[None; SIZE_BOARD]; SIZE_BOARD];
        white_pos
            .iter()
            .for_each(|&(x, y)| test_bboard[x][y] = Some(true));
        black_pos
            .iter()
            .for_each(|&(x, y)| test_bboard[x][y] = Some(false));
        let mut test_board: Board = test_bboard.into();
        println!("// Initial configuration:");
        test_board.print();
//        for i in 0..19 {
//            print!("// ");
//            for j in 0..19 {
//                match test_board[j][i] {
//                    Some(true) => print!("⊖"),
//                    Some(false) => print!("⊕"),
//                    None => print!("_"),
//                }
//            }
//            println!();
//        }
        let mut score_board = heuristic::evaluate_board(&mut test_board);
        let mut record: [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD] =
            initialize_record(&mut test_board, actual_player);
        let mut global_results: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![];
        for dir in 0..4 {
            let tmp_result = match score_board[pos2check.0][pos2check.1][dir].0 {
                4 => connect_4(
                    pos2check,
                    &mut score_board,
                    &mut test_board,
                    &mut record,
                    actual_player,
                    dir,
                ),
                _ => vec![],
            };
            if tmp_result.len() == 0 {
                continue;
            }
            tmp_result
                .iter()
                .for_each(|(defensive_move, type_of_threat, opp)| {
                    global_results.push((*defensive_move, *type_of_threat, (*opp).clone()));
                    // For each result, print the details of the threat + possible response
                    println!("\n// Details: [dir:{}]", dir);
                    for i in 0..19 {
                        print!("// ");
                        for j in 0..19 {
                            // Print specific attack move
                            if (defensive_move.0, defensive_move.1) == (j as usize, i as usize) {
                                print!("⊛")
                            } else if opp.contains(&(j, i)) {
                                print!("⊙")
                            } else {
                                match test_board.get_pawn(j, i) {
                                    Some(true) => print!("⊖"),
                                    Some(false) => print!("⊕"),
                                    None => print!("_"),
                                }
                            }
                        }
                        println!();
                    }
                    println!("// DEFENSIVE_MOVE:");

                    print!("// (({},{}), ", defensive_move.0, defensive_move.1);
                    print!(
                        "TypeOfThreat::{}, vec![",
                        match type_of_threat {
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
                    opp.iter().enumerate().for_each(|(i, (x, y))| {
                        if i == (opp.len() - 1) {
                            print!("({},{})", x, y)
                        } else {
                            print!("({},{}),", x, y)
                        }
                    });
                    println!("])");
                });
        }

        // print expected datastruct in test
        println!();
        global_results
            .iter()
            .enumerate()
            .for_each(|(j, (defensive_move, type_of_threat, opp))| {
                print!("(({},{}), ", defensive_move.0, defensive_move.1);
                print!(
                    "TypeOfThreat::{}, vec![",
                    match type_of_threat {
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
                opp.iter().enumerate().for_each(|(i, (x, y))| {
                    if i == (opp.len() - 1) {
                        print!("({},{})", x, y)
                    } else {
                        print!("({},{}),", x, y)
                    }
                });
                if j == (global_results.len() - 1) {
                    println!("])");
                } else {
                    println!("]),");
                }
            });
        global_results == expected_result
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
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

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊛_________
    // _________⊕_________
    // _________⊕_________
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
    // DEFENSIVE_MOVE:
    // ((9,4), TypeOfThreat::FourOF, vec![])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊛_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::FourOF, vec![])
    #[test]
    fn threat_connect_4_connect_4_normal() {
        let black_pos = vec![(9, 8), (9, 7), (9, 6), (9, 5)];
        let white_pos = vec![];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 4), TypeOfThreat::FiveTake, vec![]),
            ((9, 9), TypeOfThreat::FiveTake, vec![]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖⊕⊕_________
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

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊛_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖⊕⊕⊙________
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
    // DEFENSIVE_MOVE:
    // ((9,4), TypeOfThreat::FourOF, vec![(10,8)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖⊕⊕⊙________
    // _________⊛_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::FourOF, vec![(10,8)])
    #[test]
    fn threat_connect_4_catchis() {
        let black_pos = vec![(9, 8), (9, 7), (9, 6), (9, 5), (8, 8)];
        let white_pos = vec![(7, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 4), TypeOfThreat::FiveTake, vec![(10, 8)]),
            ((9, 9), TypeOfThreat::FiveTake, vec![(10, 8)]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _______⊖⊕__________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖⊕⊕_________
    // _______⊖⊕__________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _______⊖⊕⊛⊙________
    // _________⊕_________
    // _________⊕⊙________
    // _________⊕_________
    // _______⊖⊕⊕⊙________
    // _______⊖⊕__________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,4), TypeOfThreat::FourOF, vec![(10,4),(10,6),(10,8)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _______⊖⊕__________
    // _________⊕_________
    // _________⊕⊙________
    // _________⊕_________
    // _______⊖⊕⊕⊙________
    // _______⊖⊕⊛⊙________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::FourOF, vec![(10,9),(10,8),(10,6)])
    #[test]
    fn threat_connect_4_catch_extremity1() {
        let black_pos = vec![(9, 8), (9, 7), (9, 6), (9, 5), (8, 9), (8, 4), (8, 8)];
        let white_pos = vec![(7, 9), (7, 4), (7, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            (
                (9, 4),
                TypeOfThreat::FiveTake,
                vec![(10, 4), (10, 6), (10, 8)],
            ),
            (
                (9, 9),
                TypeOfThreat::FiveTake,
                vec![(10, 9), (10, 8), (10, 6)],
            ),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖⊕__________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊛_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖⊕__________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,4), TypeOfThreat::FourOF, vec![])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖⊕⊛⊙________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::FourOF, vec![(10,9)])
    #[test]
    fn threat_connect_4_catch_extremity2() {
        let black_pos = vec![(9, 8), (9, 7), (9, 6), (9, 5), (8, 9)];
        let white_pos = vec![(7, 9)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 4), TypeOfThreat::FiveTake, vec![]),
            ((9, 9), TypeOfThreat::FiveTake, vec![(10, 9)]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // __________⊕⊖_______
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊛_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // __________⊕⊖_______
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,4), TypeOfThreat::FourOF, vec![])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // ________⊙⊛⊕⊖_______
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::FourOF, vec![(8,9)])
    #[test]
    fn threat_connect_4_catch_extremity3() {
        let black_pos = vec![(9, 8), (9, 7), (9, 6), (9, 5), (10, 9)];
        let white_pos = vec![(11, 9)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 4), TypeOfThreat::FiveTake, vec![]),
            ((9, 9), TypeOfThreat::FiveTake, vec![(8, 9)]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // ________⊕_⊖________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊛_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // ________⊕_⊖________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,4), TypeOfThreat::FourOF, vec![])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _______⊙⊕⊛⊖________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::FourOF, vec![(7,9)])
    #[test]
    fn threat_connect_4_catch_extremity_hard1() {
        let black_pos = vec![(9, 8), (9, 7), (9, 6), (9, 5), (8, 9)];
        let white_pos = vec![(10, 9)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 4), TypeOfThreat::FiveTake, vec![]),
            ((9, 9), TypeOfThreat::FiveTake, vec![(7, 9)]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖⊕_⊖________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊛_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖⊕_⊖________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,4), TypeOfThreat::FourOF, vec![])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖⊕⊛⊖________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::FourOF, vec![])
    #[test]
    fn threat_connect_4_not_catch_extremity() {
        let black_pos = vec![(9, 8), (9, 7), (9, 6), (9, 5), (8, 9)];
        let white_pos = vec![(10, 9), (7, 9)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 4), TypeOfThreat::FiveTake, vec![]),
            ((9, 9), TypeOfThreat::FiveTake, vec![]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖__⊖________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊛_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖__⊖________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,4), TypeOfThreat::FourOF, vec![])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _______⊖_⊛⊖________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::FourOF, vec![])
    #[test]
    fn threat_connect_4_catch_9_in_a_row_first() {
        let black_pos = vec![
            (9, 8),
            (9, 7),
            (9, 6),
            (9, 5),
            (9, 10),
            (9, 11),
            (9, 12),
            (9, 13),
        ];
        let white_pos = vec![(10, 9), (7, 9)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 4), TypeOfThreat::FiveTake, vec![]),
            ((9, 9), TypeOfThreat::OneTake, vec![]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // ________⊕_⊖________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊛_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // ________⊕_⊖________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,4), TypeOfThreat::FourOF, vec![])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _______⊙⊕⊛⊖________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::FourOF, vec![(7,9)])
    #[test]
    fn threat_connect_4_catch_9_in_a_row_catch() {
        let black_pos = vec![
            (9, 8),
            (9, 7),
            (9, 6),
            (9, 5),
            (9, 10),
            (9, 11),
            (9, 12),
            (9, 13),
            (8, 9),
        ];
        let white_pos = vec![(10, 9)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 4), TypeOfThreat::FiveTake, vec![]),
            ((9, 9), TypeOfThreat::OneTake, vec![(7, 9)]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // ________⊕⊕⊖________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊛_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _______⊙⊕⊕⊖________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,4), TypeOfThreat::FourOF, vec![(7,8)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // ________⊕⊕⊖________
    // _________⊛_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // _________⊕_________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((9,9), TypeOfThreat::FourOF, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_catch_1() {
        let black_pos = vec![
            (9, 8),
            (9, 7),
            (9, 6),
            (9, 5),
            (9, 10),
            (9, 11),
            (9, 12),
            (9, 13),
            (8, 8),
        ];
        let white_pos = vec![(10, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((9, 4), TypeOfThreat::FiveTake, vec![(7, 8)]),
            ((9, 9), TypeOfThreat::OneTake, vec![]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (9, 7),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊛________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ⊙⊕⊕⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,4), TypeOfThreat::FourOF, vec![(0,8)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // __⊛________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,9), TypeOfThreat::FourOF, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_catch_close_border() {
        let black_pos = vec![
            (2, 8),
            (2, 7),
            (2, 6),
            (2, 5),
            (2, 10),
            (2, 11),
            (2, 12),
            (2, 13),
            (1, 8),
        ];
        let white_pos = vec![(3, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((2, 4), TypeOfThreat::FiveTake, vec![(0, 8)]),
            ((2, 9), TypeOfThreat::OneTake, vec![]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 7),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊛________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ⊙⊕⊕⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,4), TypeOfThreat::FourOF, vec![(0,8)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // __⊛________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,9), TypeOfThreat::FourOF, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_other_position_0() {
        let black_pos = vec![
            (2, 8),
            (2, 7),
            (2, 6),
            (2, 5),
            (2, 10),
            (2, 11),
            (2, 12),
            (2, 13),
            (1, 8),
        ];
        let white_pos = vec![(3, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((2, 4), TypeOfThreat::FiveTake, vec![(0, 8)]),
            ((2, 9), TypeOfThreat::OneTake, vec![]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 6),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊛________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ⊙⊕⊕⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,4), TypeOfThreat::FourOF, vec![(0,8)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // __⊛________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,9), TypeOfThreat::FourOF, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_other_position_1() {
        let black_pos = vec![
            (2, 8),
            (2, 7),
            (2, 6),
            (2, 5),
            (2, 10),
            (2, 11),
            (2, 12),
            (2, 13),
            (1, 8),
        ];
        let white_pos = vec![(3, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((2, 4), TypeOfThreat::FiveTake, vec![(0, 8)]),
            ((2, 9), TypeOfThreat::OneTake, vec![]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 5),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊛________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ⊙⊕⊕⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,4), TypeOfThreat::FourOF, vec![(0,8)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // __⊛________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,9), TypeOfThreat::FourOF, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_other_position_2() {
        let black_pos = vec![
            (2, 8),
            (2, 7),
            (2, 6),
            (2, 5),
            (2, 10),
            (2, 11),
            (2, 12),
            (2, 13),
            (1, 8),
        ];
        let white_pos = vec![(3, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((2, 4), TypeOfThreat::FiveTake, vec![(0, 8)]),
            ((2, 9), TypeOfThreat::OneTake, vec![]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 8),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // __⊛________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,9), TypeOfThreat::FourOF, vec![])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊛________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,14), TypeOfThreat::FourOF, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_other_position_3() {
        let black_pos = vec![
            (2, 8),
            (2, 7),
            (2, 6),
            (2, 5),
            (2, 10),
            (2, 11),
            (2, 12),
            (2, 13),
            (1, 8),
        ];
        let white_pos = vec![(3, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((2, 9), TypeOfThreat::OneTake, vec![]),
            ((2, 14), TypeOfThreat::FiveTake, vec![]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 10),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // __⊛________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,9), TypeOfThreat::FourOF, vec![])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊛________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,14), TypeOfThreat::FourOF, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_other_position_4() {
        let black_pos = vec![
            (2, 8),
            (2, 7),
            (2, 6),
            (2, 5),
            (2, 10),
            (2, 11),
            (2, 12),
            (2, 13),
            (1, 8),
        ];
        let white_pos = vec![(3, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((2, 9), TypeOfThreat::OneTake, vec![]),
            ((2, 14), TypeOfThreat::FiveTake, vec![]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 11),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // __⊛________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,9), TypeOfThreat::FourOF, vec![])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊛________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,14), TypeOfThreat::FourOF, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_other_position_5() {
        let black_pos = vec![
            (2, 8),
            (2, 7),
            (2, 6),
            (2, 5),
            (2, 10),
            (2, 11),
            (2, 12),
            (2, 13),
            (1, 8),
        ];
        let white_pos = vec![(3, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((2, 9), TypeOfThreat::OneTake, vec![]),
            ((2, 14), TypeOfThreat::FiveTake, vec![]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 12),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // __⊛________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,9), TypeOfThreat::FourOF, vec![])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊛________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,14), TypeOfThreat::FourOF, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_other_position_6() {
        let black_pos = vec![
            (2, 8),
            (2, 7),
            (2, 6),
            (2, 5),
            (2, 10),
            (2, 11),
            (2, 12),
            (2, 13),
            (1, 8),
        ];
        let white_pos = vec![(3, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((2, 9), TypeOfThreat::OneTake, vec![]),
            ((2, 14), TypeOfThreat::FiveTake, vec![]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 13),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _⊕_________________
    // _⊕_________________
    // _⊕_________________
    // ⊕⊕⊖________________
    // ___________________
    // _⊕_________________
    // _⊕_________________
    // _⊕_________________
    // _⊕_________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _⊛_________________
    // _⊕_________________
    // _⊕_________________
    // _⊕_________________
    // ⊕⊕⊖________________
    // ___________________
    // _⊕_________________
    // _⊕_________________
    // _⊕_________________
    // _⊕_________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((1,4), TypeOfThreat::FourOF, vec![])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _⊕_________________
    // _⊕_________________
    // _⊕_________________
    // ⊕⊕⊖________________
    // _⊛_________________
    // _⊕_________________
    // _⊕_________________
    // _⊕_________________
    // _⊕_________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((1,9), TypeOfThreat::FourOF, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_false_catch() {
        let black_pos = vec![
            (1, 8),
            (1, 7),
            (1, 6),
            (1, 5),
            (1, 10),
            (1, 11),
            (1, 12),
            (1, 13),
            (0, 8),
        ];
        let white_pos = vec![(2, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((1, 4), TypeOfThreat::FiveTake, vec![]),
            ((1, 9), TypeOfThreat::OneTake, vec![]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (1, 6),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ⊕__________________
    // ⊕__________________
    // ⊕__________________
    // ⊕⊖_________________
    // ___________________
    // ⊕__________________
    // ⊕__________________
    // ⊕__________________
    // ⊕__________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ⊛__________________
    // ⊕__________________
    // ⊕__________________
    // ⊕__________________
    // ⊕⊖_________________
    // ___________________
    // ⊕__________________
    // ⊕__________________
    // ⊕__________________
    // ⊕__________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((0,4), TypeOfThreat::FourOF, vec![])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ⊕__________________
    // ⊕__________________
    // ⊕__________________
    // ⊕⊖_________________
    // ⊛__________________
    // ⊕__________________
    // ⊕__________________
    // ⊕__________________
    // ⊕__________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((0,9), TypeOfThreat::FourOF, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_left_border() {
        let black_pos = vec![
            (0, 8),
            (0, 7),
            (0, 6),
            (0, 5),
            (0, 10),
            (0, 11),
            (0, 12),
            (0, 13),
        ];
        let white_pos = vec![(1, 8)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((0, 4), TypeOfThreat::FiveTake, vec![]),
            ((0, 9), TypeOfThreat::OneTake, vec![]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (0, 6),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // __⊛________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ⊙⊕⊕⊖_______________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,0), TypeOfThreat::FourOF, vec![(0,4)])

    // Details: [dir:3]
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // __⊛________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,5), TypeOfThreat::FourOF, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_other_close_top_0() {
        let black_pos = vec![
            (2, 4),
            (2, 3),
            (2, 2),
            (2, 1),
            (2, 6),
            (2, 7),
            (2, 8),
            (2, 9),
            (1, 4),
        ];
        let white_pos = vec![(3, 4)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((2, 0), TypeOfThreat::FiveTake, vec![(0, 4)]),
            ((2, 5), TypeOfThreat::OneTake, vec![]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 2),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // _⊕_⊖_______________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
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
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ⊙⊕⊛⊖_______________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
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
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,0), TypeOfThreat::FourOF, vec![(0,0)])

    // Details: [dir:3]
    // _⊕_⊖_______________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊛________________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,5), TypeOfThreat::FourOF, vec![])
    #[test]
    fn threat_connect_4_fake_catch_close_top_1() {
        let black_pos = vec![(2, 4), (2, 3), (2, 2), (2, 1), (1, 0)];
        let white_pos = vec![(3, 0)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((2, 0), TypeOfThreat::FiveTake, vec![(0, 0)]),
            ((2, 5), TypeOfThreat::FiveTake, vec![]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 2),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // _⊕_________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
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
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // _⊕⊛________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
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
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,0), TypeOfThreat::FourOF, vec![])

    // Details: [dir:3]
    // _⊕_________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊛________________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,5), TypeOfThreat::FourOF, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_other_close_top_2() {
        let black_pos = vec![(2, 4), (2, 3), (2, 2), (2, 1), (1, 0)];
        let white_pos = vec![];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((2, 0), TypeOfThreat::FiveTake, vec![]),
            ((2, 5), TypeOfThreat::FiveTake, vec![]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 2),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // _⊕⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // __⊕________________
    // __⊕________________
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
    // ___________________

    // Details: [dir:3]
    // _⊕⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊛________________
    // __⊕________________
    // __⊕________________
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
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,4), TypeOfThreat::FourSOF, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_other_close_top_3() {
        let black_pos = vec![(2, 6), (2, 5), (2, 3), (2, 2), (2, 1), (2, 0), (1, 0)];
        let white_pos = vec![];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((2, 4), TypeOfThreat::ThreeTake, vec![])];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 0),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // _⊕⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // __⊕________________
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
    // ___________________
    // ___________________

    // Details: [dir:3]
    // _⊕⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊛________________
    // __⊕________________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,4), TypeOfThreat::FourTake, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_other_close_top_four() {
        let black_pos = vec![(2, 5), (2, 3), (2, 2), (2, 1), (2, 0), (1, 0)];
        let white_pos = vec![];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((2, 4), TypeOfThreat::FourTake, vec![])];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 0),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // __⊖________________
    // _⊕⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
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
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // __⊖________________
    // _⊕⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊛________________
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
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,6), TypeOfThreat::FourSOF, vec![])
    #[test]
    fn threat_connect_4_fake_catch_4_in_a_row_so() {
        let black_pos = vec![(2, 5), (2, 4), (2, 3), (2, 2), (1, 2)];
        let white_pos = vec![(2, 1)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((2, 6), TypeOfThreat::FiveTake, vec![])];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 2),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ⊕__________________
    // _⊕⊖________________
    // __⊕________________
    // ___⊕_______________
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
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:0]
    // ⊕__________________
    // _⊕⊖________________
    // __⊕________________
    // ___⊕_______________
    // ____⊛______________
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
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,4), TypeOfThreat::FourSOF, vec![])
    #[test]
    fn threat_connect_4_diagonal_left_0() {
        let black_pos = vec![(0, 0), (3, 3), (1, 1), (2, 2)];
        let white_pos = vec![(2, 1)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((4, 4), TypeOfThreat::FiveTake, vec![])];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 2),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕_⊖_______________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ⊙⊕⊛⊖_______________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,9), TypeOfThreat::FourOF, vec![(0,9)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // _⊕_⊖_______________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊛________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,14), TypeOfThreat::FourOF, vec![])
    #[test]
    fn threat_connect_4_9_in_row_with_catch() {
        let black_pos = vec![
            (2, 8),
            (2, 7),
            (2, 6),
            (2, 5),
            (2, 10),
            (2, 11),
            (2, 12),
            (2, 13),
            (1, 9),
        ];
        let white_pos = vec![(3, 9)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((2, 9), TypeOfThreat::OneTake, vec![(0, 9)]),
            ((2, 14), TypeOfThreat::FiveTake, vec![]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 13),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // __⊕________________
    // _⊕⊕⊖_______________
    // _⊕⊕⊖_______________
    // _⊕_⊖_______________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊛________________
    // __⊕________________
    // ⊙_⊕________________
    // ⊙⊕⊕⊖_______________
    // ⊙⊕⊕⊖_______________
    // _⊕_⊖_______________
    // ⊙_⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,4), TypeOfThreat::FourOF, vec![(0,7),(0,6),(0,8),(0,10)])

    // Details: [dir:3]
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // __⊕________________
    // ⊙_⊕________________
    // _⊕⊕⊖_______________
    // ⊙⊕⊕⊖_______________
    // ⊙⊕⊛⊖_______________
    // ⊙_⊕________________
    // __⊕________________
    // __⊕________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,9), TypeOfThreat::FourOF, vec![(0,9),(0,6),(0,8),(0,10)])
    #[test]
    fn threat_connect_4_9_in_row_with_2_possibe_catchs() {
        let black_pos = vec![
            (2, 8),
            (2, 7),
            (2, 6),
            (2, 5),
            (2, 10),
            (2, 11),
            (2, 12),
            (1, 9),
            (1, 8),
            (1, 7),
        ];
        let white_pos = vec![(3, 9), (3, 8), (3, 7)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            (
                (2, 4),
                TypeOfThreat::FiveTake,
                vec![(0, 7), (0, 6), (0, 8), (0, 10)],
            ),
            (
                (2, 9),
                TypeOfThreat::TwoTake,
                vec![(0, 9), (0, 6), (0, 8), (0, 10)],
            ),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 7),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ⊕__________________
    // _⊕_________________
    // __⊕_⊖______________
    // ___⊕_______________
    // __⊕________________
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
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:0]
    // ⊕__________________
    // _⊕_________________
    // __⊕_⊖______________
    // ___⊕_______________
    // __⊕_⊛______________
    // _⊙_________________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,4), TypeOfThreat::FourSOF, vec![(1,5)])
    #[test]
    fn threat_connect_4_diagonal_left_catch1() {
        let black_pos = vec![(0, 0), (3, 3), (1, 1), (2, 2), (2, 4)];
        let white_pos = vec![(4, 2)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 4), TypeOfThreat::FiveTake, vec![(1, 5)]),
            // ((5,5), TypeOfThreat::FourOF, vec![])
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 2),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ⊕__________________
    // _⊕_________________
    // __⊕_⊕______________
    // ___⊕_______________
    // __⊖________________
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
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:0]
    // ⊕__________________
    // _⊕___⊙_____________
    // __⊕_⊕______________
    // ___⊕_______________
    // __⊖_⊛______________
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
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((4,4), TypeOfThreat::FourSOF, vec![(5,1)])
    #[test]
    fn threat_connect_4_diagonal_left_catch_bloup1() {
        let black_pos = vec![(0, 0), (3, 3), (1, 1), (2, 2), (4, 2)];
        let white_pos = vec![(2, 4)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 4), TypeOfThreat::FiveTake, vec![(5, 1)]),
            // ((5,5), TypeOfThreat::FourOF, vec![])
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 2),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ⊕__________________
    // _⊕_________________
    // __⊕_⊕______________
    // ___⊕_______________
    // __⊖_⊖______________
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
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn threat_connect_4_diagonal_left_catch_bloup_close_0() {
        let black_pos = vec![(0, 0), (3, 3), (1, 1), (2, 2), (4, 2)];
        let white_pos = vec![(2, 4), (4, 4)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 2),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // _⊕_________________
    // __⊕_⊕______________
    // ___⊕_______________
    // __⊖_⊕______________
    // _____⊖_____________
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
    // ___________________
    // ___________________

    // Details: [dir:0]
    // ⊛__________________
    // _⊕___⊙_____________
    // __⊕_⊕______________
    // ___⊕_______________
    // __⊖_⊕______________
    // _____⊖_____________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((0,0), TypeOfThreat::FourSOF, vec![(5,1)])
    #[test]
    fn threat_connect_4_diagonal_left_catch_bloup_close_bottom() {
        let black_pos = vec![(3, 3), (1, 1), (2, 2), (4, 4), (4, 2)];
        let white_pos = vec![(2, 4), (5, 5)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((0, 0), TypeOfThreat::FiveTake, vec![(5, 1)]),
            // ((5,5), TypeOfThreat::FourOF, vec![])
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 2),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // _⊕_⊖_______________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊖________________
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
    // ___________________
    // ___________________

    // Details: [dir:3]
    // ⊙⊕⊛⊖_______________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊕________________
    // __⊖________________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,0), TypeOfThreat::FourSOF, vec![(0,0)])
    #[test]
    fn threat_connect_4_fake_catch_close_top_2_so_0() {
        let black_pos = vec![(2, 4), (2, 3), (2, 2), (2, 1), (1, 0)];
        let white_pos = vec![(3, 0), (2, 5)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((2, 0), TypeOfThreat::FiveTake, vec![(0, 0)])];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 2),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
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
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___⊕_______________
    // __⊕________________
    // _⊕_________________
    // ⊕__________________

    // Details: [dir:2]
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
    // ___________________
    // ___________________
    // ___________________
    // ____⊛______________
    // ___⊕_______________
    // __⊕________________
    // _⊕_________________
    // ⊕__________________
    // DEFENSIVE_MOVE:
    // ((4,14), TypeOfThreat::FourSOF, vec![])
    #[test]
    fn threat_connect_4_diagonal_bottom_left_so() {
        let black_pos = vec![(0, 18), (1, 17), (2, 16), (3, 15)];
        let white_pos = vec![];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((4, 14), TypeOfThreat::FiveTake, vec![]),
            // ((2,5), TypeOfThreat::FourOF, vec![])
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (3, 15),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
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
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // _______________⊕___
    // ________________⊕__
    // _________________⊕_
    // __________________⊕

    // Details: [dir:0]
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
    // ___________________
    // ___________________
    // ___________________
    // ______________⊛____
    // _______________⊕___
    // ________________⊕__
    // _________________⊕_
    // __________________⊕
    // DEFENSIVE_MOVE:
    // ((14,14), TypeOfThreat::FourSOF, vec![])
    #[test]
    fn threat_connect_4_diagonal_bottom_right_so() {
        let black_pos = vec![(18, 18), (17, 17), (16, 16), (15, 15)];
        let white_pos = vec![];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((14, 14), TypeOfThreat::FiveTake, vec![])];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (18, 18),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // __________________⊕
    // _________________⊕_
    // ________________⊕__
    // _______________⊕___
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
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:2]
    // __________________⊕
    // _________________⊕_
    // ________________⊕__
    // _______________⊕___
    // ______________⊛____
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
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((14,4), TypeOfThreat::FourSOF, vec![])
    #[test]
    fn threat_connect_4_diagonal_up_right_so() {
        let black_pos = vec![(18, 0), (17, 1), (16, 2), (15, 3)];
        let white_pos = vec![];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((14, 4), TypeOfThreat::FiveTake, vec![])];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (16, 2),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // __________________⊕
    // _________________⊕_
    // ________________⊕__
    // _______________⊕___
    // ______________⊖____
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
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn threat_connect_4_diago_not_take() {
        let black_pos = vec![(18, 0), (17, 1), (16, 2), (15, 3)];
        let white_pos = vec![(14, 4)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (16, 2),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // _⊕___⊕_____________
    // __⊕_⊕______________
    // ___⊕_______________
    // __⊕_⊕______________
    // _____⊖_____________
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
    // ___________________
    // ___________________

    // Details: [dir:0]
    // ⊛__________________
    // _⊕___⊕_____________
    // __⊕_⊕______________
    // ___⊕_______________
    // __⊕_⊕______________
    // _____⊖_____________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((0,0), TypeOfThreat::FourSOF, vec![])
    #[test]
    fn threat_connect_4_diagonal_multiple_4_in_a_row_0() {
        let black_pos = vec![(3, 3), (1, 1), (2, 2), (4, 4), (4, 2), (2, 4), (5, 1)];
        let white_pos = vec![(5, 5)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((0, 0), TypeOfThreat::FiveTake, vec![])];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (2, 2),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // _⊕___⊕_____________
    // __⊕_⊕______________
    // ___⊕_______________
    // __⊕_⊕______________
    // _____⊖_____________
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
    // ___________________
    // ___________________

    // Details: [dir:0]
    // ⊛__________________
    // _⊕___⊕_____________
    // __⊕_⊕______________
    // ___⊕_______________
    // __⊕_⊕______________
    // _____⊖_____________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((0,0), TypeOfThreat::FourSOF, vec![])

    // Details: [dir:2]
    // ___________________
    // _⊕___⊕_____________
    // __⊕_⊕______________
    // ___⊕_______________
    // __⊕_⊕______________
    // _⊛___⊖_____________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((1,5), TypeOfThreat::FourOF, vec![])

    // Details: [dir:2]
    // ______⊛____________
    // _⊕___⊕_____________
    // __⊕_⊕______________
    // ___⊕_______________
    // __⊕_⊕______________
    // _____⊖_____________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((6,0), TypeOfThreat::FourOF, vec![])
    #[test]
    fn threat_connect_4_diagonal_multiple_4_in_a_row_1() {
        let black_pos = vec![(3, 3), (1, 1), (2, 2), (4, 4), (4, 2), (2, 4), (5, 1)];
        let white_pos = vec![(5, 5)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((0, 0), TypeOfThreat::FiveTake, vec![]),
            ((1, 5), TypeOfThreat::FiveTake, vec![]),
            ((6, 0), TypeOfThreat::FiveTake, vec![]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (3, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ______⊕____________
    // _⊕___⊕_____________
    // __⊕_⊕______________
    // ___⊕_______________
    // ____⊕______________
    // _____⊖_____________
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
    // ___________________
    // ___________________

    // Details: [dir:0]
    // ⊛_____⊕____________
    // _⊕___⊕_____________
    // __⊕_⊕______________
    // ___⊕_______________
    // ____⊕______________
    // _____⊖_____________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((0,0), TypeOfThreat::FourSOF, vec![])

    // Details: [dir:2]
    // ______⊕____________
    // _⊕___⊕_____________
    // __⊕_⊕______________
    // ___⊕_______________
    // __⊛_⊕______________
    // _____⊖_____________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((2,4), TypeOfThreat::FourSOF, vec![])
    #[test]
    fn threat_connect_4_diagonal_multiple_4_in_a_row_2() {
        let black_pos = vec![(3, 3), (1, 1), (2, 2), (4, 4), (4, 2), (5, 1), (6, 0)];
        let white_pos = vec![(5, 5)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((0, 0), TypeOfThreat::FiveTake, vec![]),
            ((2, 4), TypeOfThreat::FiveTake, vec![]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (3, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ⊖_____⊖____________
    // _⊕___⊕_____________
    // __⊕_⊕______________
    // ___⊕_______________
    // ____⊕______________
    // _____⊖_____________
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
    // ___________________
    // ___________________
    #[test]
    fn threat_connect_4_diagonal_multiple_4_in_a_row_3() {
        let black_pos = vec![(3, 3), (1, 1), (2, 2), (4, 4), (4, 2), (5, 1)];
        let white_pos = vec![(5, 5), (6, 0), (0, 0)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (3, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ⊖_____⊖____________
    // _⊕___⊕_____________
    // __⊕_⊕______________
    // ___⊕_______________
    // __⊕_⊕______________
    // _____⊖_____________
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
    // ___________________
    // ___________________

    // Details: [dir:2]
    // ⊖_____⊖____________
    // _⊕___⊕_____________
    // __⊕_⊕______________
    // ___⊕_______________
    // __⊕_⊕______________
    // _⊛___⊖_____________
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
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((1,5), TypeOfThreat::FourSOF, vec![])
    #[test]
    fn threat_connect_4_diagonal_multiple_4_in_a_row_4() {
        let black_pos = vec![(3, 3), (1, 1), (2, 2), (4, 4), (4, 2), (5, 1), (2, 4)];
        let white_pos = vec![(5, 5), (6, 0), (0, 0)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> =
            vec![((1, 5), TypeOfThreat::FiveTake, vec![])];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (3, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // ____⊕___⊕__________
    // _____⊕_⊕___________
    // ______⊕____________
    // _______⊕___________
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
    fn threat_connect_4_diagonal_multiple_4_in_a_row_5() {
        let black_pos = vec![(5, 5), (6, 6), (7, 7), (4, 4), (9, 3), (8, 4), (7, 5)];
        let white_pos = vec![];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (3, 3),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // ____⊕___⊕_⊕________
    // _____⊕_⊕___⊖_______
    // ______⊕____________
    // _______⊕___________
    // ______⊕____________
    // _____⊖_____________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:0]
    // ___________________
    // ___________________
    // ___________________
    // ___⊛_____⊕_________
    // ____⊕___⊕_⊕________
    // _____⊕_⊕___⊖_______
    // ______⊕_⊙__________
    // _______⊕___________
    // ______⊕____________
    // _____⊖_____________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((3,3), TypeOfThreat::FourOF, vec![(8,6)])

    // Details: [dir:0]
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // ____⊕___⊕_⊕________
    // _____⊕_⊕___⊖_______
    // ______⊕_⊙__________
    // _______⊕___________
    // ______⊕_⊛__________
    // _____⊖_____________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((8,8), TypeOfThreat::FourOF, vec![(8,6)])
    #[test]
    fn threat_connect_4_diagonal_multiple_4_in_a_row_6() {
        let black_pos = vec![
            (5, 5),
            (6, 6),
            (7, 7),
            (4, 4),
            (9, 3),
            (8, 4),
            (7, 5),
            (6, 8),
            (10, 4),
        ];
        let white_pos = vec![(5, 9), (11, 5)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((3, 3), TypeOfThreat::FiveTake, vec![(8, 6)]),
            ((8, 8), TypeOfThreat::FiveTake, vec![(8, 6)]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (4, 4),
            Some(false),
            expected_result
        ))
    }

    // Initial configuration:
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // ____⊕___⊕_⊕________
    // _____⊕_⊕___⊖_______
    // ______⊕____________
    // _______⊕___________
    // ______⊕____________
    // _____⊖_____________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________

    // Details: [dir:0]
    // ___________________
    // ___________________
    // ___________________
    // ___⊛_____⊕_________
    // ____⊕___⊕_⊕________
    // _____⊕_⊕___⊖_______
    // ______⊕_⊙__________
    // _______⊕___________
    // ______⊕____________
    // _____⊖_____________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((3,3), TypeOfThreat::FourOF, vec![(8,6)])

    // Details: [dir:0]
    // ___________________
    // ___________________
    // ___________________
    // _________⊕_________
    // ____⊕___⊕_⊕________
    // _____⊕_⊕___⊖_______
    // ______⊕_⊙__________
    // _______⊕___________
    // ______⊕_⊛__________
    // _____⊖_____________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((8,8), TypeOfThreat::FourOF, vec![(8,6)])

    // Details: [dir:2]
    // ___________________
    // ___________________
    // ________⊙__________
    // _________⊕_________
    // ____⊕___⊕_⊕________
    // _____⊕_⊕___⊖_______
    // ______⊕____________
    // _____⊛_⊕___________
    // ______⊕____________
    // _____⊖_____________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((5,7), TypeOfThreat::FourOF, vec![(8,2)])

    // Details: [dir:2]
    // ___________________
    // ___________________
    // ________⊙_⊛________
    // _________⊕_________
    // ____⊕___⊕_⊕________
    // _____⊕_⊕___⊖_______
    // ______⊕____________
    // _______⊕___________
    // ______⊕____________
    // _____⊖_____________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // DEFENSIVE_MOVE:
    // ((10,2), TypeOfThreat::FourOF, vec![(8,2)])
    #[test]
    fn threat_connect_4_diagonal_multiple_4_in_a_row_7() {
        let black_pos = vec![
            (5, 5),
            (6, 6),
            (7, 7),
            (4, 4),
            (9, 3),
            (8, 4),
            (7, 5),
            (6, 8),
            (10, 4),
        ];
        let white_pos = vec![(5, 9), (11, 5)];
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![
            ((3, 3), TypeOfThreat::FiveTake, vec![(8, 6)]),
            ((8, 8), TypeOfThreat::FiveTake, vec![(8, 6)]),
            ((5, 7), TypeOfThreat::FiveTake, vec![(8, 2)]),
            ((10, 2), TypeOfThreat::FiveTake, vec![(8, 2)]),
        ];
        assert!(test_threat_4(
            white_pos,
            black_pos,
            (6, 6),
            Some(false),
            expected_result
        ))
    }

    #[test]
    fn test_record_connect_2() {
        let black_pos: Vec<(usize, usize)> = vec![(5, 3), (5, 5), (5, 6)];
        let white_pos: Vec<(usize, usize)> = vec![(5, 7)];

        let mut test_bboard: [[Option<bool>; SIZE_BOARD]; SIZE_BOARD] =
            [[None; SIZE_BOARD]; SIZE_BOARD];
        white_pos
            .iter()
            .for_each(|&(x, y)| test_bboard[x][y] = Some(true));
        black_pos
            .iter()
            .for_each(|&(x, y)| test_bboard[x][y] = Some(false));
        // Print initial configuration
        let mut test_board: Board = test_bboard.into();
        println!("// Initial configuration:");
        for i in 0..19 {
            print!("// ");
            for j in 0..19 {
                match test_board.get_pawn(j, i){
                    Some(true) => print!("⊖"),
                    Some(false) => print!("⊕"),
                    None => print!("_"),
                }
            }
            println!();
        }
        let mut score_board = heuristic::evaluate_board(&mut test_board);

        let mut record: [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD] =
            initialize_record(&mut test_board, Some(false));
        println!("Initial record : ");
        for i in 0..19 {
            for j in 0..19 {
                for dir in 0..4 {
                    match record[j][i][dir] {
                        true => print!("T"),
                        false => print!("F"),
                    }
                }
                print!(" ");
            }
            println!();
            println!();
        }
        let ret = connect_2(
            &mut test_board,
            &mut score_board,
            &mut record,
            Some(false),
            (5, 5),
            3,
        );
        println!("found {} results", ret.len());
        println!("Record after connect_2 : ");
        for i in 0..19 {
            for j in 0..19 {
                for dir in 0..4 {
                    match record[j][i][dir] {
                        true => print!("T"),
                        false => print!("F"),
                    }
                }
                print!(" ");
            }
            println!();
            println!();
        }
        //assert!(false)
        assert!(true)
    }
    #[test]
    fn test_record_connect_3() {
        let black_pos: Vec<(usize, usize)> = vec![(9, 7), (9, 8), (9, 9)];
        let white_pos: Vec<(usize, usize)> = vec![(9, 10)];

        let mut test_bboard: [[Option<bool>; SIZE_BOARD]; SIZE_BOARD] =
            [[None; SIZE_BOARD]; SIZE_BOARD];
        white_pos
            .iter()
            .for_each(|&(x, y)| test_bboard[x][y] = Some(true));
        black_pos
            .iter()
            .for_each(|&(x, y)| test_bboard[x][y] = Some(false));
        // Print initial configuration
        let mut test_board: Board = test_bboard.into();
        println!("// Initial configuration:");
//        for i in 0..19 {
//            print!("// ");
//            for j in 0..19 {
//                match test_board[j][i] {
//                    Some(true) => print!("⊖"),
//                    Some(false) => print!("⊕"),
//                    None => print!("_"),
//                }
//            }
//            println!();
//        }
        let mut score_board = heuristic::evaluate_board(&mut test_board);

        let mut record: [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD] =
            initialize_record(&mut test_board, Some(false));
        println!("Initial record : ");
        for i in 0..19 {
            for j in 0..19 {
                for dir in 0..4 {
                    match record[j][i][dir] {
                        true => print!("T"),
                        false => print!("F"),
                    }
                }
                print!(" ");
            }
            println!();
            println!();
        }
        let ret = connect_3(
            &mut test_board,
            &mut score_board,
            &mut record,
            Some(false),
            (9, 7),
            3,
        );
        println!("found {} results", ret.len());
        println!("Record after connect_3 : ");
        for i in 0..19 {
            for j in 0..19 {
                for dir in 0..4 {
                    match record[j][i][dir] {
                        true => print!("T"),
                        false => print!("F"),
                    }
                }
                print!(" ");
            }
            println!();
            println!();
        }
        //assert!(false)
        assert!(true)
    }
    #[test]
    fn test_record_connect_4() {
        let black_pos: Vec<(usize, usize)> = vec![(5, 5), (5, 6), (5, 7), (5, 8)];
        let white_pos: Vec<(usize, usize)> = vec![];

        let mut test_bboard: [[Option<bool>; SIZE_BOARD]; SIZE_BOARD] =
            [[None; SIZE_BOARD]; SIZE_BOARD];
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
//        for i in 0..19 {
//            print!("// ");
//            for j in 0..19 {
//                match test_board[j][i] {
//                    Some(true) => print!("⊖"),
//                    Some(false) => print!("⊕"),
//                    None => print!("_"),
//                }
//            }
//            println!();
//        }
        let mut score_board = heuristic::evaluate_board(&mut test_board);

        let mut record: [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD] =
            initialize_record(&mut test_board, Some(false));
        println!("Initial record : ");
        for i in 0..19 {
            for j in 0..19 {
                for dir in 0..4 {
                    match record[j][i][dir] {
                        true => print!("T"),
                        false => print!("F"),
                    }
                }
                print!(" ");
            }
            println!();
            println!();
        }
        let ret = connect_4(
            (5, 8),
            &mut score_board,
            &mut test_board,
            &mut record,
            Some(false),
            3,
        );
        println!("found {} results", ret.len());
        println!("Record after connect_4 : ");
        for i in 0..19 {
            for j in 0..19 {
                for dir in 0..4 {
                    match record[j][i][dir] {
                        true => print!("T"),
                        false => print!("F"),
                    }
                }
                print!(" ");
            }
            println!();
            println!();
        }
        //assert!(false)
        assert!(true)
    }
}
