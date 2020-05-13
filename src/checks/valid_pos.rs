use super::super::model::game;

pub fn all_except(valid_pos: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    let mut ret: Vec<(usize, usize)> = vec![];
    for i in 0..19 {
        for j in 0..19 {
            if valid_pos.iter().any(|&index| index == (i,j)) {
                continue;
            } else {
                ret.push((i,j));
            }
        }
    }
    ret
}

pub fn valid_pos(game: &mut game::Game, line: usize, col: usize) -> bool {
    !(line >= 19 || col >= 19 || game.forbidden.iter().any(|&point| point == (line, col)))
}
