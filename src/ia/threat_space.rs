use super::super::checks::after_turn_check::DIRECTIONS;
use super::super::checks::capture::DIRS;
use super::super::checks::double_three::check_double_three_hint;
use super::super::render::board::SIZE_BOARD;
// use super::handle_board::*;
use super::heuristic;
use super::get_ia;

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

// macro_rules! valid_coord_explore {
//     (
//         $x: expr,
//         $y: expr
//     ) => {
//         $x > 0 && $x < (SIZE_BOARD as isize - 1) && $y > 0 && $y < (SIZE_BOARD as isize - 1)
//     };
// }

macro_rules! get_opp {
    ($e:expr) => {
        match $e {
            Some(a) => Some(!a),
            _ => unreachable!(),
        }
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
            // println!("je check autre dirr");
            let (mut new_line, mut new_col):(isize, isize) = (x as isize, y as isize);
            // println!(
            //     // "dir: (x:{},y:{}):[{}-{}-{}]/dir:{}",
            //     x,
            //     y,
            //     score_board[x][y][new_dir].0,
            //     match score_board[x as usize][y as usize][new_dir].1 {
            //         Some(true) => "true",
            //         Some(false) => "false",
            //         None => "none",
            //     },
            //     match score_board[x as usize][y as usize][new_dir].2 {
            //         Some(true) => "true",
            //         Some(false) => "false",
            //         None => "none",
            //     },
            //     new_dir
            // );
            match score_board[x][y][new_dir] {
                // (2, Some(true), Some(false)) | (2, None, Some(false)) => {
                (2, Some(true), Some(false)) => {
                    // println!("ici");
                    explore_align_light!(board, new_line, new_col, actual_player, new_dir, 1);
                    coordinates.push((new_line as usize, new_col as usize));
                },
                // (2, Some(false), Some(true)) | (2, Some(false), None) => {
                (2, Some(false), Some(true)) => {
                    // println!("la");
                    explore_align_light!(board, new_line, new_col, actual_player, new_dir, -1);
                    coordinates.push((new_line as usize, new_col as usize));
                },
                (0, Some(false), Some(false)) => {
                    // println!("YOOOOOOOOOOYHHHHHHH");
                    let (mut new_line2, mut new_col2):(isize, isize) = (x as isize, y as isize);
                    let (mut new_line3, mut new_col3):(isize, isize) = (x as isize, y as isize);
                    explore_one!(new_line2, new_col2, new_dir, -1);
                    explore_one!(new_line3, new_col3, new_dir, 1);
                    
                    let (mut new_line2,mut new_col2) = match (valid_coord_uniq!(new_line2), valid_coord_uniq!(new_col2)) {
                        (true, true) => (new_line2, new_col2),
                        (false, true) => (x as isize, new_col2),
                        (true, false) => (new_line2, y as isize),
                        (false, false) => (x as isize, y as isize)
                    };

                    let (mut new_line3,mut new_col3) = match (valid_coord_uniq!(new_line3), valid_coord_uniq!(new_col3)) {
                        (true, true) => (new_line3, new_col3),
                        (false, true) => (x as isize, new_col3),
                        (true, false) => (new_line3, y as isize),
                        (false, false) => (x as isize, y as isize)
                    };

                    // println!(
                    //     "debug_extremity_line2: ({}-{}-{})|dir:{}|(x/y):({}-{})",
                    //     score_board[new_line2 as usize][new_col2 as usize][new_dir].0,
                    //     match score_board[new_line2 as usize][new_col2 as usize][new_dir].1 {
                    //         Some(true) => "true",
                    //         Some(false) => "false",
                    //         None => "none",
                    //     },
                    //     match score_board[new_line2 as usize][new_col2 as usize][new_dir].2 {
                    //         Some(true) => "true",
                    //         Some(false) => "false",
                    //         None => "none",
                    //     },
                    //     new_dir,
                    //     new_line2,
                    //     new_col2
                    // );
                    // println!(
                    //     "debug_extremity_line3: ({}-{}-{})|dir:{}|(x/y):({}-{})",
                    //     score_board[new_line3 as usize][new_col3 as usize][new_dir].0,
                    //     match score_board[new_line3 as usize][new_col3 as usize][new_dir].1 {
                    //         Some(true) => "true",
                    //         Some(false) => "false",
                    //         None => "none",
                    //     },
                    //     match score_board[new_line3 as usize][new_col3 as usize][new_dir].2 {
                    //         Some(true) => "true",
                    //         Some(false) => "false",
                    //         None => "none",
                    //     },
                    //     new_dir,
                    //     new_line3,
                    //     new_col3
                    // );
                    
                    // match score_board[new_line2 as usize][new_col2 as usize][new_dir] {
                    //     (1, Some(true), Some(false)) => {
                    //         coordinates.push((new_line3 as usize, new_col3 as usize));
                    //     },
                    //     _ => (),
                    // }
                    // match score_board[new_line3 as usize][new_col3 as usize][new_dir] {
                    //     (1, Some(false), Some(true)) => {
                    //         coordinates.push((new_line2 as usize, new_col2 as usize));
                    //     },
                    //     _ => (),
                    // }
                    match (
                            score_board[new_line2 as usize][new_col2 as usize][new_dir],
                            score_board[new_line3 as usize][new_col3 as usize][new_dir]
                        ){
                            ((1, Some(true), Some(false)), (0, Some(false), Some(false))) => {
                                coordinates.push((new_line3 as usize, new_col3 as usize));
                            },
                            ((0, Some(false), Some(false)), (1, Some(false), Some(true))) => {
                                coordinates.push((new_line2 as usize, new_col2 as usize));
                            },
                            ((1, Some(false), Some(false)), (1, Some(false), Some(false))) => {
                                let opp = get_opp!(actual_player);
                                if opp == board[new_line2 as usize][new_col2 as usize]
                                    && board[new_line3 as usize][new_col3 as usize] == actual_player {
                                        explore_one!(new_line3, new_col3, new_dir, 1);
                                        coordinates.push((new_line3 as usize, new_col3 as usize));
                                } else if board[new_line2 as usize][new_col2 as usize] == actual_player
                                    && opp == board[new_line3 as usize][new_col3 as usize] {
                                    explore_one!(new_line2, new_col2, new_dir, -1);
                                    coordinates.push((new_line2 as usize, new_col2 as usize));
                                }
                            },
                            _ => (),
                        }
                },

                // (0, Some(false), Some(true)) => {
                //     println!("OUIIIIIIKLLLLLLE");
                //     let (mut new_line2, mut new_col2):(isize, isize) = (x as isize, y as isize);
                //     let (mut new_line3, mut new_col3):(isize, isize) = (x as isize, y as isize);
                //     explore_one!(new_line2, new_col2, dir, 1);
                //     match score_board[new_line2 as usize][new_col2 as usize][new_dir] {
                //         (1, Some(false), Some(true)) => {
                //             explore_align_light!(board, new_line, new_col, actual_player, new_dir, -1);
                //             coordinates.push((new_line as usize, new_col as usize));
                //         },
                //         _ => (),
                //     }
                // },
                // (0, Some(true), Some(false)) => {
                //     println!("JUJUJUJUJUJUJUJUJUJUJU");
                //     explore_one!(new_line2, new_col2, dir, 1);
                //     match score_board[new_line2 as usize][new_col2 as usize][new_dir] {
                //         (1, Some(false), Some(true)) => {
                //             explore_align_light!(board, new_line, new_col, actual_player, new_dir, -1);
                //             coordinates.push((new_line as usize, new_col as usize));
                //         },
                //         _ => (),
                //     }
                // },
                _ => (),
            }
            // Check for extremities
            // let (mut new_line2, mut new_col2):(isize, isize) = (x as isize, y as isize);
            // explore_one!(new_line2, new_col2, dir, 1);
            // // match (score_board[x][y][new_dir], )
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
     let (mut ccline, mut cccol) = (*cline, *ccol);
    //  println!("LIMITTTEEEEE: {}", limit);
    let mut tmp_positions: Vec<Vec<(usize,usize)>> = vec![];
    for expansion in 0..limit {
        // println!("expansion->({},{})", ccline, cccol);
        tmp_positions.push(
            capture_coordinates(
                score_board,
                board,
                actual_player,
                ccline as usize,
                cccol as usize,
                dir
            )
        );
        // println!("multiplication: [{}-{}]/{}", DIRECTIONS[dir].0,DIRECTIONS[dir].1,orientation * expansion as isize);
        explore_one!(ccline, cccol, dir, orientation);
     }
     // DEBUG
    //  println!("EXPLORE AND FIND THREATS");
     for x in 0..tmp_positions.len() {
        //  println!("New position");
        for y in 0..tmp_positions[x].len() {
            // println!("({},{})", tmp_positions[x][y].0, tmp_positions[x][y].1);
        }
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
    // println!("DEBUG_YOUHOU-new: ({}:{})", new_line, new_col);
    let (mut nline, mut ncol) = (new_line, new_col);
    explore_one!(nline, ncol, dir, way);
    let (cline, ccol) = (new_line, new_col);
    // println!("DEBUG_YOUHOU: ({}:{})", nline, ncol);
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
                                dir,
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
        // println!("WINFORSURE");
        explore_and_find_threats(score_board, board, 5, opp_way, &cline, &ccol, threat, actual_player, dir, all_threats, &new_line, &new_col);
    }
}

fn connect_4(
    (line, col): (usize, usize),
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    record: &mut [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD],
    actual_player: Option<bool>,
    // actual_take: &mut isize,
    dir: usize
) -> Vec<((usize,usize), TypeOfThreat, Vec<(usize,usize)>)> {
    let mut new_line: isize = line as isize;
    let mut new_col: isize = col as isize;
    let mut all_threats:Vec<((usize,usize), TypeOfThreat, Vec<(usize,usize)>)> = vec![];

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
        // (Some(true),Some(false)) | (None,Some(false)) => { println!("oulala"); manage_so(score_board, board, record, actual_player, dir, new_line, new_col, 1, -1, TypeOfThreat::FOUR_SO, &mut all_threats); all_threats },
        (Some(true),Some(false)) | (None,Some(false)) => { manage_so(score_board, board, record, actual_player, dir, new_line, new_col, 1, -1, TypeOfThreat::FOUR_SO, &mut all_threats); all_threats },
        // (Some(false),Some(true)) | (Some(false),None) => { println!("ouille ouille ouille"); manage_so(score_board, board, record, actual_player, dir, new_line, new_col, -1, 1, TypeOfThreat::FOUR_SO, &mut all_threats); all_threats },
        (Some(false),Some(true)) | (Some(false),None) => { manage_so(score_board, board, record, actual_player, dir, new_line, new_col, -1, 1, TypeOfThreat::FOUR_SO, &mut all_threats); all_threats },
        (Some(false),Some(false)) => {
            let new_line2: isize = line as isize;
            let new_col2: isize = col as isize;
            manage_so(score_board, board, record, actual_player, dir, new_line, new_col, -1, 1, TypeOfThreat::FOUR_O, &mut all_threats);
            manage_so(score_board, board, record, actual_player, dir, new_line2, new_col2, 1, -1, TypeOfThreat::FOUR_O, &mut all_threats);
            all_threats
            },
        _ => { all_threats },
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

pub fn threat_connect_4_search_space(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    actual_player: Option<bool>,
    actual_take: &mut isize,
) -> () {
    // 1. Initialize datastructures storing ready to be checked positions as well as threats
    let mut record: [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD] = initialize_record(board, score_board, actual_player);

    // 1.2. Initialize Threat board -> Vec containing with_capacity data (3 avrg max_possible threats per position) | (4 max defensive) 
    // Optimized version of : [[Vec<(enum, Vec<(usize,usize)>)>; SIZE_BOARD]; SIZE_BOARD]
    let mut threat_connect_4_board: Vec<Vec<Vec<(TypeOfThreat, Vec<(usize,usize)>)>>> = (0..SIZE_BOARD).map(|_|
                                                                                    (0..SIZE_BOARD).map(|_| 
                                                                                        Vec::with_capacity(AVRG_MAX_MULTIPLE_THREATS)
                                                                                    ).collect()
                                                                                ).collect();



    // 2. Parse board for actual_player's pawns
    for line in 0..SIZE_BOARD {
        for col in 0..SIZE_BOARD {
            if board[line][col] == actual_player {
                for dir in 0..4 {
                    // println!("passe5fois");
                    // let ret: Vec<((usize,usize), TypeOfThreat, Vec<(usize,usize)>)> = 
                    let x = match score_board[line][col][dir].0 {
                        5 => vec![], //Instant win ?
                        // if record[x][y][dir]
                        4 => { connect_4((line, col), score_board, board, &mut record, actual_player, dir) },
                        3 => vec![],
                        2 => vec![],
                        _ => vec![],
                    };
                    // if not empty inside, ppush
                    // x.iter().for_each(|((x,y), typeOfThreat, Opp)| Opp.iter().for_each(|(x,y)| println!("opp: ({},{})", x, y)));
                    x.iter().for_each(|((x,y), typeOfThreat, Opp)| threat_connect_4_board[*x][*y].push((*typeOfThreat, Opp.clone()))); // check borrow issue here !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                    // ret.iter().for_each(|&((x,y), typeOfThreat, Opp)| threat_connect_4_board[x][y].push((typeOfThreat, Opp)));
                }
                ()
            }
            // else if board[line][col] == opposite_player {
                // 1. Check if opposant == 2 (on peut manger et gagner)
                // 2. Check if opposant == 4 (regarder si on peut perdre, avec 4 prises)
            // } 
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


    // Version last connect_4

    // fn test_threat(
    //     white_pos: Vec<(usize, usize)>,
    //     black_pos: Vec<(usize, usize)>,
    //     actual_take: &mut isize,
    //     opp_take: &mut isize,
    //     pos2check: (usize, usize),
    //     actual_player: Option<bool>,
    //     expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)>
    // ) -> bool {
    //     let mut test_board = [[None; SIZE_BOARD]; SIZE_BOARD];
    //     white_pos
    //         .iter()
    //         .for_each(|&(x, y)| test_board[x][y] = Some(true));
    //     black_pos
    //         .iter()
    //         .for_each(|&(x, y)| test_board[x][y] = Some(false));
    //     for i in 0..19 {
    //         for j in 0..19 {
    //             match test_board[j][i] {
    //                 Some(true) => print!("⊖"),
    //                 Some(false) => print!("⊕"),
    //                 None => print!("_"),
    //             }
    //         }
    //         println!();
    //     }
    //     let mut score_board = heuristic::evaluate_board(&mut test_board);
    //     let mut record: [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD] = initialize_record(&mut test_board, &mut score_board, actual_player);
    //     // let mut threat_connect_4_board: Vec<Vec<Vec<(TypeOfThreat, Vec<(usize,usize)>)>>> = (0..SIZE_BOARD).map(|_|
    //                                                                             //         (0..SIZE_BOARD).map(|_| 
    //                                                                             //             Vec::with_capacity(AVRG_MAX_MULTIPLE_THREATS)
    //                                                                             //         ).collect()
    //                                                                             //  ).collect();

    //     let mut tmp_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![];
    //     // for i in 0..19 {
    //     //     for j in 0..19 {
    //     //         match test_board[j][i] {
    //     //             Some(true) => print!("B"),
    //     //             Some(false) => print!("N"),
    //     //             None => print!("E"),
    //     //         }
    //     //         score_board[j][i].iter().for_each(|&(value, a, b)| {
    //     //             print!("{:2}{}{}", value, get_bool!(a), get_bool!(b))
    //     //         });
    //     //         print!(" ");
    //     //     }
    //     //     println!();
    //     // }
    //     for dir in 0..4 {
    //         tmp_result = match score_board[pos2check.0][pos2check.1][dir].0 {
    //             4 => { connect_4(pos2check, &mut score_board, &mut test_board, &mut record, actual_player, dir) },
    //             _ => { vec![] }
    //         };
    //         println!("DEBUT°°°DEBUG_CONNECT: len({})", tmp_result.len());
    //         // tmp_result.iter().for_each(|((x,y), typeOfThreat, Opp)| {  threat_connect_4_board[*x][*y].push((*typeOfThreat, Opp.clone())) } );
    //         // ret_debug.iter().for_each(|&x| print!("{}", x)); 
    //         println!("DEBUG_CONNECT: len({})", tmp_result.len());

    //         tmp_result.iter().for_each(|(defensive_move, type_of_threat, opp)| {
    //             println!("-----------------");
    //             println!("DEFENSIVE_MOVE-TMP-RESULT:");
    //             println!("({},{})", defensive_move.0, defensive_move.1);
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

    //             expected_result.iter().for_each(|(defensive_move, type_of_threat, opp)| {
    //                 println!("-----------------");
    //                 println!("DEFENSIVE_MOVE_EXPECTED:");
    //                 println!("({},{})", defensive_move.0, defensive_move.1);
    //                 println!("typeOfThreat:");
    //                 match type_of_threat {
    //                     TypeOfThreat::FIVE => println!("FIVE"),
    //                     TypeOfThreat::FIVE_TAKE => println!("FIVE_TAKE"),
    //                     TypeOfThreat::FOUR_O => println!("FOUR_O"),
    //                     TypeOfThreat::FOUR_SO => println!("FOUR_SO"),
    //                     TypeOfThreat::TAKE => println!("TAKE"),
    //                     TypeOfThreat::THREE_O => println!("THREE_O"),
    //                 }
    //                 println!("Responses:");
    //                 opp.iter().for_each(|(x,y)| println!("({},{})", x, y));
    //             });
    //             if tmp_result != vec![] {
    //                 break ;
    //             }
    //     }

    //         // println!("DEBUG_CONNECT: len({})", threat_connect_4_board.len());

    //         // threat_connect_4_board.iter().enumerate().for_each(|(i_x, x)| {
    //         //     println!("i_x: {}", i_x);
    //         //     x.iter().enumerate().for_each(|(i_y, y)| {
    //         //         println!("i_y: {}", i_y);
    //         //         println!("y_len: {}", y.len());
    //         //         y.iter().for_each(|(type_of_threat, opp)| {
    //         //             println!("-----------------");
    //         //             println!("DEFENSIVE_MOVE:");
    //         //             println!("({},{})", i_x, i_y);
    //         //             println!("typeOfThreat:");
    //         //             match type_of_threat {
    //         //                 TypeOfThreat::FIVE => println!("FIVE"),
    //         //                 TypeOfThreat::FIVE_TAKE => println!("FIVE_TAKE"),
    //         //                 TypeOfThreat::FOUR_O => println!("FOUR_O"),
    //         //                 TypeOfThreat::FOUR_SO => println!("FOUR_SO"),
    //         //                 TypeOfThreat::TAKE => println!("TAKE"),
    //         //                 TypeOfThreat::THREE_O => println!("THREE_O"),
    //         //             }
    //         //             println!("Responses:");
    //         //             opp.iter().for_each(|(x,y)| println!("({},{})", x, y));
    //         //         });
    //         //     });
    //         // });
    //         tmp_result == expected_result
    // }

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
        let mut record: [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD] = initialize_record(&mut test_board, &mut score_board, actual_player);
        // let mut threat_connect_4_board: Vec<Vec<Vec<(TypeOfThreat, Vec<(usize,usize)>)>>> = (0..SIZE_BOARD).map(|_|
                                                                                //         (0..SIZE_BOARD).map(|_| 
                                                                                //             Vec::with_capacity(AVRG_MAX_MULTIPLE_THREATS)
                                                                                //         ).collect()
                                                                                //  ).collect();

        let mut tmp_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = vec![];
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
            tmp_result = match score_board[pos2check.0][pos2check.1][dir].0 {
                4 => { connect_4(pos2check, &mut score_board, &mut test_board, &mut record, actual_player, dir) },
                // 3 => { connect_4(pos2check, &mut score_board, &mut test_board, &mut record, actual_player, actual_take, dir) },
                // 2 => { println!("OUPS_I_DID_IT_AGAIN") ; connect_2(&mut test_board, &mut score_board, &mut record, actual_player, pos2check, dir) }
                // 2 => { connect_2(&mut test_board, &mut score_board, &mut record, actual_player, pos2check, dir) }
                _ => { vec![] }
            };
            if tmp_result.len() == 0 {
                continue ;
            }
            // tmp_result = connect_2(&mut test_board, &mut score_board, &mut record, actual_player, pos2check, 3);
            // println!("DEBUT°°°DEBUG_CONNECT: len({})", tmp_result.len());
            // tmp_result.iter().for_each(|((x,y), typeOfThreat, Opp)| {  threat_connect_4_board[*x][*y].push((*typeOfThreat, Opp.clone())) } );
            // ret_debug.iter().for_each(|&x| print!("{}", x)); 
            // println!("DEBUG_CONNECT: len({})", tmp_result.len());
            tmp_result.iter().for_each(|(defensive_move, type_of_threat, opp)| {
                global_results.push((*defensive_move, *type_of_threat, (*opp).clone()));
                // println!("-----------------");
                // For each result, print the details of the threat + possible response
                println!("\n// Details: [dir:{}]", dir);
                for i in 0..19 {
                    print!("// ");
                    for j in 0..19 {
                        // Print specific attack move
                        if (defensive_move.0, defensive_move.1) == (j as usize,i as usize) {
                            print!("⊛")
                        } else if opp.contains(&(j,i)) {
                            print!("⊙")
                        }
                        else {
                            match test_board[j][i] {
                                Some(true) => print!("⊖"),
                                Some(false) => print!("⊕"),
                                None => print!("_"),
                            }
                        }
                    }
                    println!();
                }
                // ((9,4), TypeOfThreat::FOUR_O, vec![(10,8)]),
                println!("// DEFENSIVE_MOVE:");

                print!("// (({},{}), ", defensive_move.0, defensive_move.1);
                print!("TypeOfThreat::{}, vec![", match type_of_threat {
                    TypeOfThreat::FIVE => "FIVE",
                    TypeOfThreat::FIVE_TAKE => "FIVE_TAKE",
                    TypeOfThreat::FOUR_O => "FOUR_O",
                    TypeOfThreat::FOUR_SO => "FOUR_SO",
                    TypeOfThreat::TAKE => "TAKE",
                    TypeOfThreat::THREE_O => "THREE_O",
                });
                opp.iter().enumerate().for_each(|(i,(x,y))| if i == (opp.len() - 1)  { print!("({},{})", x, y) } else { print!("({},{}),", x, y) });
                println!("])");
            });
                
        }
        
        // print expected datastruct in test
        println!();
        global_results.iter().enumerate().for_each(|(j,(defensive_move, type_of_threat, opp))| { 
                print!("(({},{}), ", defensive_move.0, defensive_move.1);
                print!("TypeOfThreat::{}, vec![", match type_of_threat {
                TypeOfThreat::FIVE => "FIVE",
                TypeOfThreat::FIVE_TAKE => "FIVE_TAKE",
                TypeOfThreat::FOUR_O => "FOUR_O",
                TypeOfThreat::FOUR_SO => "FOUR_SO",
                TypeOfThreat::TAKE => "TAKE",
                TypeOfThreat::THREE_O => "THREE_O",
            });
            opp.iter().enumerate().for_each(|(i,(x,y))| if i == (opp.len() - 1)  { print!("({},{})", x, y) } else { print!("({},{}),", x, y) });
            if j == (global_results.len() - 1) { println!("])"); } else { println!("]),"); }
        });
            // println!("DEBUG_CONNECT: len({})", threat_connect_4_board.len());

            // threat_connect_4_board.iter().enumerate().for_each(|(i_x, x)| {
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
    // ((9,4), TypeOfThreat::FOUR_O, vec![])

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
    // ((9,9), TypeOfThreat::FOUR_O, vec![])
    #[test]
    fn threat_connect_4_connect_4_normal() {
        let mut black_pos = vec![(9,8),(9,7), (9,6), (9,5)];
        let white_pos = vec![];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((9,4), TypeOfThreat::FOUR_O, vec![]),
                ((9,9), TypeOfThreat::FOUR_O, vec![])
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
    // ((9,4), TypeOfThreat::FOUR_O, vec![(10,8)])

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
    // ((9,9), TypeOfThreat::FOUR_O, vec![(10,8)])
    #[test]
    fn threat_connect_4_catchis() {
        let mut black_pos = vec![(9,8),(9,7), (9,6), (9,5), (8,8)];
        let white_pos = vec![(7,8)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((9,4), TypeOfThreat::FOUR_O, vec![(10,8)]),
                ((9,9), TypeOfThreat::FOUR_O, vec![(10,8)])
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
    // ((9,4), TypeOfThreat::FOUR_O, vec![(10,4),(10,6),(10,8)])

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
    // ((9,9), TypeOfThreat::FOUR_O, vec![(10,9),(10,8),(10,6)])
    #[test]
    fn threat_connect_4_catch_extremity1() {
        let mut black_pos = vec![(9,8),(9,7), (9,6), (9,5), (8,9), (8,4), (8,8)];
        let white_pos = vec![(7,9), (7,4), (7,8)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((9,4), TypeOfThreat::FOUR_O, vec![(10,4),(10,6),(10,8)]),
                ((9,9), TypeOfThreat::FOUR_O, vec![(10,9),(10,8),(10,6)])
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
    // ((9,4), TypeOfThreat::FOUR_O, vec![])

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
    // ((9,9), TypeOfThreat::FOUR_O, vec![(10,9)])
    #[test]
    fn threat_connect_4_catch_extremity2() {
        let mut black_pos = vec![(9,8),(9,7), (9,6), (9,5), (8,9)];
        let white_pos = vec![(7,9)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((9,4), TypeOfThreat::FOUR_O, vec![]),
                ((9,9), TypeOfThreat::FOUR_O, vec![(10,9)])
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
    // ((9,4), TypeOfThreat::FOUR_O, vec![])

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
    // ((9,9), TypeOfThreat::FOUR_O, vec![(8,9)])
    #[test]
    fn threat_connect_4_catch_extremity3() {
        let mut black_pos = vec![(9,8),(9,7), (9,6), (9,5), (10,9)];
        let white_pos = vec![(11,9)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((9,4), TypeOfThreat::FOUR_O, vec![]),
                ((9,9), TypeOfThreat::FOUR_O, vec![(8,9)])
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
    // ((9,4), TypeOfThreat::FOUR_O, vec![])

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
    // ((9,9), TypeOfThreat::FOUR_O, vec![(7,9)])
    #[test]
    fn threat_connect_4_catch_extremity_hard1() {
        let mut black_pos = vec![(9,8),(9,7), (9,6), (9,5), (8,9)];
        let white_pos = vec![(10,9)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((9,4), TypeOfThreat::FOUR_O, vec![]),
                ((9,9), TypeOfThreat::FOUR_O, vec![(7,9)])
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
    // ((9,4), TypeOfThreat::FOUR_O, vec![])

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
    // ((9,9), TypeOfThreat::FOUR_O, vec![])
    #[test]
    fn threat_connect_4_not_catch_extremity() {
        let mut black_pos = vec![(9,8),(9,7), (9,6), (9,5), (8,9)];
        let white_pos = vec![(10,9), (7,9)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((9,4), TypeOfThreat::FOUR_O, vec![]),
                ((9,9), TypeOfThreat::FOUR_O, vec![])
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
    // ((9,4), TypeOfThreat::FOUR_O, vec![])

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
    // ((9,9), TypeOfThreat::FOUR_O, vec![])
    #[test]
    fn threat_connect_4_catch_9_in_a_row_first() {
        let mut black_pos = vec![(9,8),(9,7),(9,6),(9,5),(9,10),(9,11),(9,12),(9,13)];
        let white_pos = vec![(10,9), (7,9)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((9,4), TypeOfThreat::FOUR_O, vec![]),
                ((9,9), TypeOfThreat::FOUR_O, vec![])
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
    // ((9,4), TypeOfThreat::FOUR_O, vec![])

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
    // ((9,9), TypeOfThreat::FOUR_O, vec![(7,9)])
    #[test]
    fn threat_connect_4_catch_9_in_a_row_catch() {
        let mut black_pos = vec![(9,8),(9,7),(9,6),(9,5),(9,10),(9,11),(9,12),(9,13),(8,9)];
        let white_pos = vec![(10,9)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((9,4), TypeOfThreat::FOUR_O, vec![]),
                ((9,9), TypeOfThreat::FOUR_O, vec![(7,9)])
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
    // ((9,4), TypeOfThreat::FOUR_O, vec![(7,8)])

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
    // ((9,9), TypeOfThreat::FOUR_O, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_catch_1() {
        let mut black_pos = vec![(9,8),(9,7),(9,6),(9,5),(9,10),(9,11),(9,12),(9,13),(8,8)];
        let white_pos = vec![(10,8)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((9,4), TypeOfThreat::FOUR_O, vec![(7,8)]),
                ((9,9), TypeOfThreat::FOUR_O, vec![])
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
    // ((2,4), TypeOfThreat::FOUR_O, vec![(0,8)])

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
    // ((2,9), TypeOfThreat::FOUR_O, vec![])
    #[test]
    fn _fake_catch_9_in_a_row_catch_close_border() {
        let mut black_pos = vec![(2,8),(2,7),(2,6),(2,5),(2,10),(2,11),(2,12),(2,13),(1,8)];
        let white_pos = vec![(3,8)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((2,4), TypeOfThreat::FOUR_O, vec![(0,8)]),
                ((2,9), TypeOfThreat::FOUR_O, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((2,4), TypeOfThreat::FOUR_O, vec![(0,8)])

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
    // ((2,9), TypeOfThreat::FOUR_O, vec![])
    #[test]
    fn _fake_catch_9_in_a_row_other_position_0() {
        let mut black_pos = vec![(2,8),(2,7),(2,6),(2,5),(2,10),(2,11),(2,12),(2,13),(1,8)];
        let white_pos = vec![(3,8)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((2,4), TypeOfThreat::FOUR_O, vec![(0,8)]),
                ((2,9), TypeOfThreat::FOUR_O, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((2,4), TypeOfThreat::FOUR_O, vec![(0,8)])

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
    // ((2,9), TypeOfThreat::FOUR_O, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_other_position_1() {
        let mut black_pos = vec![(2,8),(2,7),(2,6),(2,5),(2,10),(2,11),(2,12),(2,13),(1,8)];
        let white_pos = vec![(3,8)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((2,4), TypeOfThreat::FOUR_O, vec![(0,8)]),
                ((2,9), TypeOfThreat::FOUR_O, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((2,4), TypeOfThreat::FOUR_O, vec![(0,8)])

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
    // ((2,9), TypeOfThreat::FOUR_O, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_other_position_2() {
        let mut black_pos = vec![(2,8),(2,7),(2,6),(2,5),(2,10),(2,11),(2,12),(2,13),(1,8)];
        let white_pos = vec![(3,8)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((2,4), TypeOfThreat::FOUR_O, vec![(0,8)]),
                ((2,9), TypeOfThreat::FOUR_O, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((2,9), TypeOfThreat::FOUR_O, vec![])

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
    // ((2,14), TypeOfThreat::FOUR_O, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_other_position_3() {
        let mut black_pos = vec![(2,8),(2,7),(2,6),(2,5),(2,10),(2,11),(2,12),(2,13),(1,8)];
        let white_pos = vec![(3,8)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((2,9), TypeOfThreat::FOUR_O, vec![]),
                ((2,14), TypeOfThreat::FOUR_O, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((2,9), TypeOfThreat::FOUR_O, vec![])

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
    // ((2,14), TypeOfThreat::FOUR_O, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_other_position_4() {
        let mut black_pos = vec![(2,8),(2,7),(2,6),(2,5),(2,10),(2,11),(2,12),(2,13),(1,8)];
        let white_pos = vec![(3,8)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((2,9), TypeOfThreat::FOUR_O, vec![]),
                ((2,14), TypeOfThreat::FOUR_O, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((2,9), TypeOfThreat::FOUR_O, vec![])

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
    // ((2,14), TypeOfThreat::FOUR_O, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_other_position_5() {
        let mut black_pos = vec![(2,8),(2,7),(2,6),(2,5),(2,10),(2,11),(2,12),(2,13),(1,8)];
        let white_pos = vec![(3,8)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((2,9), TypeOfThreat::FOUR_O, vec![]),
                ((2,14), TypeOfThreat::FOUR_O, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((2,9), TypeOfThreat::FOUR_O, vec![])

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
    // ((2,14), TypeOfThreat::FOUR_O, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_other_position_6() {
        let mut black_pos = vec![(2,8),(2,7),(2,6),(2,5),(2,10),(2,11),(2,12),(2,13),(1,8)];
        let white_pos = vec![(3,8)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((2,9), TypeOfThreat::FOUR_O, vec![]),
                ((2,14), TypeOfThreat::FOUR_O, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((1,4), TypeOfThreat::FOUR_O, vec![])

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
    // ((1,9), TypeOfThreat::FOUR_O, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_false_catch() {
        let mut black_pos = vec![(1,8),(1,7),(1,6),(1,5),(1,10),(1,11),(1,12),(1,13),(0,8)];
        let white_pos = vec![(2,8)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((1,4), TypeOfThreat::FOUR_O, vec![]),
                ((1,9), TypeOfThreat::FOUR_O, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((0,4), TypeOfThreat::FOUR_O, vec![])

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
    // ((0,9), TypeOfThreat::FOUR_O, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_left_border() {
        let mut black_pos = vec![(0,8),(0,7),(0,6),(0,5),(0,10),(0,11),(0,12),(0,13)];
        let white_pos = vec![(1,8)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((0,4), TypeOfThreat::FOUR_O, vec![]),
                ((0,9), TypeOfThreat::FOUR_O, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((2,0), TypeOfThreat::FOUR_O, vec![(0,4)])

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
    // ((2,5), TypeOfThreat::FOUR_O, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_other_close_top_0() {
        let mut black_pos = vec![(2,4),(2,3),(2,2),(2,1),(2,6),(2,7),(2,8),(2,9),(1,4)];
        let white_pos = vec![(3,4)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((2,0), TypeOfThreat::FOUR_O, vec![(0,4)]),
                ((2,5), TypeOfThreat::FOUR_O, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((2,0), TypeOfThreat::FOUR_O, vec![(0,0)])

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
    // ((2,5), TypeOfThreat::FOUR_O, vec![])
    #[test]
    fn threat_connect_4_fake_catch_close_top_1() {
        let mut black_pos = vec![(2,4),(2,3),(2,2),(2,1),(1,0)];
        let white_pos = vec![(3,0)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((2,0), TypeOfThreat::FOUR_O, vec![(0,0)]),
                ((2,5), TypeOfThreat::FOUR_O, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((2,0), TypeOfThreat::FOUR_O, vec![])

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
    // ((2,5), TypeOfThreat::FOUR_O, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_other_close_top_2() {
        let mut black_pos = vec![(2,4),(2,3),(2,2),(2,1),(1,0)];
        let white_pos = vec![];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((2,0), TypeOfThreat::FOUR_O, vec![]),
                ((2,5), TypeOfThreat::FOUR_O, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((2,4), TypeOfThreat::FOUR_SO, vec![])
    #[test]
    fn threat_connect_4_fake_catch_9_in_a_row_other_close_top_3() {
        let mut black_pos = vec![(2,6),(2,5),(2,3),(2,2),(2,1),(2,0),(1,0)];
        let white_pos = vec![];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((2,4), TypeOfThreat::FOUR_SO, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((2,6), TypeOfThreat::FOUR_SO, vec![])
    #[test]
    fn threat_connect_4_fake_catch_4_in_a_row_so() {
        let mut black_pos = vec![(2,5),(2,4),(2,3),(2,2),(1,2)];
        let white_pos = vec![(2,1)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((2,6), TypeOfThreat::FOUR_SO, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((4,4), TypeOfThreat::FOUR_SO, vec![])
    #[test]
    fn threat_connect_4_diagonal_left_0() {
        let black_pos = vec![(0,0),(3,3),(1,1),(2,2)];
        let white_pos = vec![(2,1)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((4,4), TypeOfThreat::FOUR_SO, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((2,9), TypeOfThreat::FOUR_O, vec![(0,9)])

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
    // ((2,14), TypeOfThreat::FOUR_O, vec![])
    #[test]
    fn threat_connect_4_9_in_row_with_catch() {
        let mut black_pos = vec![(2,8),(2,7),(2,6),(2,5),(2,10),(2,11),(2,12),(2,13),(1,9)];
        let white_pos = vec![(3,9)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((2,9), TypeOfThreat::FOUR_O, vec![(0,9)]),
                ((2,14), TypeOfThreat::FOUR_O, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((2,4), TypeOfThreat::FOUR_O, vec![(0,7),(0,6),(0,8),(0,10)])

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
    // ((2,9), TypeOfThreat::FOUR_O, vec![(0,9),(0,6),(0,8),(0,10)])
    #[test]
    fn threat_connect_4_9_in_row_with_2_possibe_catchs() {
        let mut black_pos = vec![(2,8),(2,7),(2,6),(2,5),(2,10),(2,11),(2,12),(1,9),(1,8),(1,7)];
        let white_pos = vec![(3,9),(3,8),(3,7)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((2,4), TypeOfThreat::FOUR_O, vec![(0,7),(0,6),(0,8),(0,10)]),
                ((2,9), TypeOfThreat::FOUR_O, vec![(0,9),(0,6),(0,8),(0,10)]),
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((4,4), TypeOfThreat::FOUR_SO, vec![(1,5)])
    #[test]
    fn threat_connect_4_diagonal_left_catch1() {
        let black_pos = vec![(0,0),(3,3),(1,1),(2,2),(2,4)];
        let white_pos = vec![(4,2)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((4,4), TypeOfThreat::FOUR_SO, vec![(1,5)]),
                // ((5,5), TypeOfThreat::FOUR_O, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((4,4), TypeOfThreat::FOUR_SO, vec![(5,1)])
    #[test]
    fn threat_connect_4_diagonal_left_catch_bloup1() {
        let black_pos = vec![(0,0),(3,3),(1,1),(2,2),(4,2)];
        let white_pos = vec![(2,4)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((4,4), TypeOfThreat::FOUR_SO, vec![(5,1)]),
                // ((5,5), TypeOfThreat::FOUR_O, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
        let black_pos = vec![(0,0),(3,3),(1,1),(2,2),(4,2)];
        let white_pos = vec![(2,4),(4,4)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((0,0), TypeOfThreat::FOUR_SO, vec![(5,1)])
    #[test]
    fn threat_connect_4_diagonal_left_catch_bloup_close_bottom() {
        let black_pos = vec![(3,3),(1,1),(2,2),(4,4),(4,2)];
        let white_pos = vec![(2,4),(5,5)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((0,0), TypeOfThreat::FOUR_SO, vec![(5,1)]),
                // ((5,5), TypeOfThreat::FOUR_O, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((2,0), TypeOfThreat::FOUR_SO, vec![(0,0)])
    #[test]
    fn threat_connect_4_fake_catch_close_top_2_SO_0() {
        let mut black_pos = vec![(2,4),(2,3),(2,2),(2,1),(1,0)];
        let white_pos = vec![(3,0),(2,5)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((2,0), TypeOfThreat::FOUR_SO, vec![(0,0)])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((4,14), TypeOfThreat::FOUR_SO, vec![])
    #[test]
    fn threat_connect_4_diagonal_bottom_left_SO() {
        let mut black_pos = vec![(0,18),(1,17),(2,16),(3,15)];
        let white_pos = vec![];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((4,14), TypeOfThreat::FOUR_SO, vec![]),
                // ((2,5), TypeOfThreat::FOUR_O, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
            (3,15),
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
    // ((14,14), TypeOfThreat::FOUR_SO, vec![])
    #[test]
    fn threat_connect_4_diagonal_bottom_right_SO() {
        let mut black_pos = vec![(18,18),(17,17),(16,16),(15,15)];
        let white_pos = vec![];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((14,14), TypeOfThreat::FOUR_SO, vec![]),
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((14,4), TypeOfThreat::FOUR_SO, vec![])
    #[test]
    fn threat_connect_4_diagonal_up_right_SO() {
        let mut black_pos = vec![(18,0),(17,1),(16,2),(15,3)];
        let white_pos = vec![]; 
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((14,4), TypeOfThreat::FOUR_SO, vec![]),
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
        let mut black_pos = vec![(18,0),(17,1),(16,2),(15,3)];
        let white_pos = vec![(14,4)]; 
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((0,0), TypeOfThreat::FOUR_SO, vec![])
    #[test]
    fn threat_connect_4_diagonal_multiple_4_in_a_row_0() {
        let black_pos = vec![(3,3),(1,1),(2,2),(4,4),(4,2),(2,4),(5,1)];
        let white_pos = vec![(5,5)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((0,0), TypeOfThreat::FOUR_SO, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((0,0), TypeOfThreat::FOUR_SO, vec![])

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
    // ((1,5), TypeOfThreat::FOUR_O, vec![])

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
    // ((6,0), TypeOfThreat::FOUR_O, vec![])
    #[test]
    fn threat_connect_4_diagonal_multiple_4_in_a_row_1() {
        let black_pos = vec![(3,3),(1,1),(2,2),(4,4),(4,2),(2,4),(5,1)];
        let white_pos = vec![(5,5)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((0,0), TypeOfThreat::FOUR_SO, vec![]),
                ((1,5), TypeOfThreat::FOUR_O, vec![]),
                ((6,0), TypeOfThreat::FOUR_O, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((0,0), TypeOfThreat::FOUR_SO, vec![])

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
    // ((2,4), TypeOfThreat::FOUR_SO, vec![])
    #[test]
    fn threat_connect_4_diagonal_multiple_4_in_a_row_2() {
        let black_pos = vec![(3,3),(1,1),(2,2),(4,4),(4,2),(5,1),(6,0)];
        let white_pos = vec![(5,5)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((0,0), TypeOfThreat::FOUR_SO, vec![]),
                ((2,4), TypeOfThreat::FOUR_SO, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
        let black_pos = vec![(3,3),(1,1),(2,2),(4,4),(4,2),(5,1)];
        let white_pos = vec![(5,5),(6,0),(0,0)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((1,5), TypeOfThreat::FOUR_SO, vec![])
    #[test]
    fn threat_connect_4_diagonal_multiple_4_in_a_row_4() {
        let black_pos = vec![(3,3),(1,1),(2,2),(4,4),(4,2),(5,1),(2,4)];
        let white_pos = vec![(5,5),(6,0),(0,0)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((1,5), TypeOfThreat::FOUR_SO, vec![])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
        let black_pos = vec![(5,5),(6,6),(7,7),(4,4),(9,3),(8,4),(7,5)];
        let white_pos = vec![];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((3,3), TypeOfThreat::FOUR_O, vec![(8,6)])

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
    // ((8,8), TypeOfThreat::FOUR_O, vec![(8,6)])
    #[test]
    fn threat_connect_4_diagonal_multiple_4_in_a_row_6() {
        let black_pos = vec![(5,5),(6,6),(7,7),(4,4),(9,3),(8,4),(7,5),(6,8),(10,4)];
        let white_pos = vec![(5,9),(11,5)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((3,3), TypeOfThreat::FOUR_O, vec![(8,6)]),
                ((8,8), TypeOfThreat::FOUR_O, vec![(8,6)])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
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
    // ((3,3), TypeOfThreat::FOUR_O, vec![(8,6)])

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
    // ((8,8), TypeOfThreat::FOUR_O, vec![(8,6)])

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
    // ((5,7), TypeOfThreat::FOUR_O, vec![(8,2)])

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
    // ((10,2), TypeOfThreat::FOUR_O, vec![(8,2)])
    #[test]
    fn threat_connect_4_diagonal_multiple_4_in_a_row_7() {
        let black_pos = vec![(5,5),(6,6),(7,7),(4,4),(9,3),(8,4),(7,5),(6,8),(10,4)];
        let white_pos = vec![(5,9),(11,5)];
        let mut white_take = 0_isize;
        let mut black_take = 0_isize;
        let expected_result: Vec<((usize, usize), TypeOfThreat, Vec<(usize, usize)>)> = 
            vec![
                ((3,3), TypeOfThreat::FOUR_O, vec![(8,6)]),
                ((8,8), TypeOfThreat::FOUR_O, vec![(8,6)]),
                ((5,7), TypeOfThreat::FOUR_O, vec![(8,2)]),
                ((10,2), TypeOfThreat::FOUR_O, vec![(8,2)])
            ];
        assert!(test_threat(
            white_pos,
            black_pos,
            &mut white_take,
            &mut black_take,
            (6, 6),
            Some(false),
            expected_result
        ))
    }

}