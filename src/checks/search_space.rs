use super::super::model::game;
use super::after_turn_check;
// use super::capture;

// fn around_area(
//     game: &mut game::Game,
//     (line, col): (isize, isize),
//     (dir_line, dir_col): (isize, isize),
// ) -> Vec<(usize, usize)> {
//     let mut ret = vec![];
//     for i in [-1, 1].iter() {
//         if capture::valid_dir(&(line, col), (i * dir_line, i * dir_col), 1) {
//             let (new_index_line, new_index_col) = (line + dir_line * 1 * i, col + dir_col * 1 * i);
//             match game.board[new_index_line as usize][new_index_col as usize] {
//                 None => ret.push((new_index_line as usize, new_index_col as usize)),
//                 _ => (),
//             }
//         }
//     }
//     ret
// }

//
pub fn search_space(game: &mut game::Game) -> Vec<(usize, usize)> {
    let mut ret = vec![];
    for line in 0..19 as isize {
        for col in 0..19 as isize {
            if game.board[line as usize][col as usize] == None {
                'outer: for &(dir_line, dir_col) in after_turn_check::DIRECTIONS.iter() {
                    for i in [-1, 1].iter() {
                        let (new_index_line, new_index_col) =
                            (line + dir_line * i, col + dir_col * i);
                        match (new_index_line, new_index_col) {
                            (x, y) if x >= 0 && y >= 0 && x < 19 && y < 19 => {
                                match game.board[x as usize][y as usize] {
                                    Some(_) => {
                                        ret.push((line as usize, col as usize));
                                        break 'outer;
                                    }
                                    _ => (),
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }
        }
    }
    // Remove forbidden moves
    game.forbidden.iter().for_each(|&x| ret.retain(|&y| y != x));
    // debug
    //    println!("---------------------------------");
    //    println!("size:{}", ret.len());
    ret.iter().for_each(|&(x, y)| match game.board[x][y] {
        None => (),
        //            None => print!("{}:{}//", x, y),
        _ => unreachable!(),
    });
    //    println!();
    // fin debug
    ret
}
pub fn search_space_hint(table: &mut [[Option<bool>; 19]; 19]) -> Vec<(usize, usize)> {
    let mut ret = vec![];
    for line in 0..19 as isize {
        for col in 0..19 as isize {
            if table[line as usize][col as usize] == None {
                'outer: for &(dir_line, dir_col) in after_turn_check::DIRECTIONS.iter() {
                    for i in [-1, 1].iter() {
                        let (new_index_line, new_index_col) =
                            (line + dir_line * i, col + dir_col * i);
                        match (new_index_line, new_index_col) {
                            (x, y) if x >= 0 && y >= 0 && x < 19 && y < 19 => {
                                match table[x as usize][y as usize] {
                                    Some(_) => {
                                        ret.push((line as usize, col as usize));
                                        break 'outer;
                                    }
                                    _ => (),
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }
        }
    }
    // Remove forbidden moves
    //    game.forbidden.iter().for_each(|&x| ret.retain(|&y| y != x));
    //    // debug
    //    //    println!("---------------------------------");
    //    //    println!("size:{}", ret.len());
    //    ret.iter().for_each(|&(x, y)| match game.board[x][y] {
    //        None => (),
    //        //            None => print!("{}:{}//", x, y),
    //        _ => unreachable!(),
    //    });
    //    println!();
    // fin debug
    ret
}
