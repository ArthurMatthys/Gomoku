use super::super::model::game;

pub fn valid_pos(game: &mut game::Game, index: usize) -> bool {
    !(index >= 361
        || game
            .forbidden
            .iter()
            .any(|&point| point.is_equal_from_index(index)))
}
