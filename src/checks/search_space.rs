use super::super::model::game;
use super::after_turn_check;
use super::capture;

fn around_area(game: &mut game::Game, (line, col): (isize,isize), (dir_line, dir_col): (isize, isize)) -> Vec<(usize,usize)> {
    let mut ret = vec![];
    for i in [-1, 1].iter() {
        if capture::valid_dir(&(line, col), (i * dir_line, i * dir_col), 1) {
            let (new_index_line,new_index_col) = (line + dir_line * 1 * i, col + dir_col * 1 * i);
            match game.board[new_index_line as usize][new_index_col as usize] {
                None => ret.push((new_index_line as usize, new_index_col as usize)),
                _ => ()
            }
        }
    }
    ret
}

// 
pub fn search_space(game: &mut game::Game) -> Vec<(usize, usize)> {
    let mut ret = vec![];
    for i in 0..19 {
        for j in 0..19 {
            if game.board[i][j] == None {
                continue;
            } else {
                after_turn_check::DIRECTIONS.iter().for_each(|&x| {
                    around_area(game, (i as isize, j as isize), x).iter().for_each(|&y| ret.push(y))
                });
            }
        }
    }
    ret
}

