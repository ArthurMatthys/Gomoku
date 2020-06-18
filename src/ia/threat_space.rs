use super::super::checks::after_turn_check::DIRECTIONS;
use super::super::checks::capture::DIRS;
use super::super::checks::double_three::check_double_three_hint;
use super::super::render::board::SIZE_BOARD;
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
        while valid_coord!($new_line, $new_col) &&
            $board[$new_line as usize][$new_col as usize] == $actual_player {
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
        while valid_coord!($new_line, $new_col) &&
            $board[$new_line as usize][$new_col as usize] == $actual_player {
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
        ($new_line + (DIRECTIONS[$dir].0 * $orientation), $new_col + (DIRECTIONS[$dir].1 * $orientation))  
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
    TAKE
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
            let (mut new_line, mut new_col):(isize, isize) = (x as isize, y as isize);
            
            match score_board[x][y][new_dir] {
                (2, Some(true), Some(false)) | (2, None, Some(false)) => {
                    explore_align_light!(board, new_line, new_col, actual_player, new_dir, 1);
                    coordinates.push((new_line as usize, new_col as usize));
                },
                (2, Some(false), Some(true)) | (2, Some(false), None) => {
                    explore_align_light!(board, new_line, new_col, actual_player, new_dir, -1);
                    coordinates.push((new_line as usize, new_col as usize));
                },
                _ => continue,
            }
        }
    }
    coordinates
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
    all_threats:&mut Vec<((usize,usize), TypeOfThreat, Vec<(usize,usize)>)>,
    new_line: &isize,
    new_col: &isize,
 )  -> () {
    let mut tmp_positions: Vec<Vec<(usize,usize)>> = vec![];
    for expansion in 0..limit {
        tmp_positions.push(
            capture_coordinates(
                score_board,
                board,
                actual_player,
                *cline as usize,
                *ccol as usize,
                dir
            )
        );
        explore_one!(cline, ccol, dir, orientation);
     }
     all_threats.push(
        (
            (*new_line as usize, *new_col as usize),
            threat,
            flatten!(tmp_positions)
        )
    );
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
    way:isize,
    opp_way: isize,
    threat: TypeOfThreat,
    all_threats:&mut Vec<((usize,usize), TypeOfThreat, Vec<(usize,usize)>)>,
) -> () {
    explore_align!(board, record, new_line, new_col, actual_player, dir, way);
    let (nline, ncol) = explore_one!(new_line, new_col, dir, way);
    let (mut cline, mut ccol) = (new_line, new_col);
    // retrieve defensive moves
    if valid_coord!(nline, ncol) && board[nline as usize][ncol as usize] == actual_player
        && score_board[nline as usize][ncol as usize][dir].0 > 0 {
            match score_board[nline as usize][ncol as usize][dir].0 {
                x if x >= 5 => {  }, // WIN FOR SURE!!!!!!!!
                4 => { 
                    all_threats.push(
                        (
                            (new_line as usize, new_col as usize),
                            threat,
                            capture_coordinates(
                                score_board,
                                board,
                                actual_player,
                                cline as usize,
                                ccol as usize,
                                dir
                            )
                        )
                    );
                 },
                3 => explore_and_find_threats(score_board, board, 2, opp_way, &cline, &ccol, threat, actual_player, dir, all_threats, &new_line, &new_col),
                2 => explore_and_find_threats(score_board, board, 3, opp_way, &cline, &ccol, threat, actual_player, dir, all_threats, &new_line, &new_col),
                1 => explore_and_find_threats(score_board, board, 4, opp_way, &cline, &ccol, threat, actual_player, dir, all_threats, &new_line, &new_col),
                _ => unreachable!()
            }  
    } else {
        explore_and_find_threats(score_board, board, 5, opp_way, &cline, &ccol, threat, actual_player, dir, all_threats, &new_line, &new_col);
    }
}

fn connect_4(
    (line, col): (usize, usize),
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    record: &mut [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD],
    actual_player: Option<bool>,
    actual_take: &mut isize,
    dir: usize
) -> Option<Vec<((usize,usize), TypeOfThreat, Vec<(usize,usize)>)>> {
    let mut new_line: isize = line as isize;
    let mut new_col: isize = col as isize;
    let mut all_threats:Vec<((usize,usize), TypeOfThreat, Vec<(usize,usize)>)> = vec![];

    if record[line][col][dir] {
        match (score_board[line][col][dir].1, score_board[line][col][dir].2) {
            // score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
            // board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
            // record: &mut [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD],
            // actual_player: Option<bool>,
            // dir: usize,
            // new_line: isize,
            // new_col: isize,
            // way: isize,
            // opp_way: isize,
            // threat: TypeOfThreat,
            // all_threats: &mut Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)>,
            (Some(true),Some(false)) | (None,Some(false)) => { manage_so(score_board, board, record, actual_player, dir, new_line, new_col, -1, 1, TypeOfThreat::FOUR_SO, &mut all_threats); Some(all_threats) },
            (Some(false),Some(true)) | (Some(false),None) => { manage_so(score_board, board, record, actual_player, dir, new_line, new_col, 1, -1, TypeOfThreat::FOUR_SO, &mut all_threats); Some(all_threats) },
            (Some(false),Some(false)) => {
                let mut new_line2: isize = line as isize;
                let mut new_col2: isize = col as isize;
                manage_so(score_board, board, record, actual_player, dir, new_line, new_col, -1, 1, TypeOfThreat::FOUR_O, &mut all_threats);
                manage_so(score_board, board, record, actual_player, dir, new_line2, new_col2, -1, 1, TypeOfThreat::FOUR_O, &mut all_threats);
                Some(all_threats)
             },
            _ => { None },
        }
    } else { None }
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

pub fn threat_search_space(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    actual_player: Option<bool>,
    actual_take: &mut isize,
) -> () {
    // 1. Initialize datastructures storing ready to be checked positions as well as threats
    let mut record: [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD] = initialize_record(board, score_board, actual_player);

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
                        4 => {  match connect_4((line, col), score_board, board, &mut record, actual_player, actual_take, dir ) {
                                    None => (),
                                    Some(x) => x.iter().for_each(|((x,y), typeOfThreat, Opp)| threat_board[*x][*y].push((*typeOfThreat, Opp.clone()))), // check borrow issue here !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                                }
                            },
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

    fn test_threat(
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
        let score_board = heuristic::evaluate_board(&mut test_board);
        for i in 0..19 {
            for j in 0..19 {
                match test_board[j][i] {
                    Some(true) => print!("B"),
                    Some(false) => print!("N"),
                    None => print!("E"),
                }
                score_board[j][i].iter().for_each(|&(value, a, b)| {
                    print!("{:2}{}{}", value, get_bool!(a), get_bool!(b))
                });
                print!(" ");
            }
            println!();
        }
        // debug
        true
        // Retrieve the wanted threat

        // Compare output with given 


    }
    #[test]
    fn win_take0() {
        let black_pos = vec![];
        let white_pos = vec![];
        let mut white_take = 0_isize;
        let mut black_take = 5_isize;
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take
        ))
    }

}