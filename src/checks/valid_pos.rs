use super::super::model::game;

pub fn all_except(valid_pos: Vec<usize>) -> Vec<usize> {
    let mut ret: Vec<usize> = vec![];
    for i in 0..361 {
        if valid_pos.iter().any(|&index| index == i) {
            continue;
        } else {
            ret.push(i);
        }
    }
    ret
}

pub fn valid_pos(game: &mut game::Game, index: usize) -> bool {
    !(index >= 361 || game.forbidden.iter().any(|&point| point == index))
}
