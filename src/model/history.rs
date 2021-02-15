use super::super::render::board;

const MULTIPLIER: i8 = 20;

/// Datastructure representing the History table.
/// It will be updated that way :
/// -> Score = [Player][x_from][y_from][x_to][y_to]
pub fn initialize_htable() -> [[[i32; board::SIZE_BOARD]; board::SIZE_BOARD]; 2] {
    [[[0_i32; board::SIZE_BOARD]; board::SIZE_BOARD]; 2]
}

/// Function that inplace sorts the silent moves in the alphabeta
/// according to their History Table results
pub fn sort_silent_moves(
    table: &[[[i32; board::SIZE_BOARD]; board::SIZE_BOARD]; 2],
    player: usize,
    silent_moves: &mut Vec<(usize, usize, i64)>
) -> () {
    silent_moves.iter_mut().for_each(| mut pos | {
        pos.2 = table[player][pos.0][pos.1] as i64;
    });
    silent_moves.sort_by(|(_, _, score1), (_,  _, score2)| score2.cmp(score1));
}

/// Function that updates (appreciate or depreciates) History scores according to their importance
pub fn update_htable(
    table: &mut [[[i32; board::SIZE_BOARD]; board::SIZE_BOARD]; 2],
    silent_moves: &[(usize, usize, i64)],
    player: usize,
    best_move: &(usize, usize, i64),
    current_depth: &i8
) -> () {
    // Depreciate score unnecessary moves
    silent_moves.iter().for_each(| &(x, y, _) | {
        table[player][x][y] -= (current_depth * current_depth * MULTIPLIER) as i32
        - (table[player][x][y] * ((current_depth * current_depth * MULTIPLIER) as i32).abs() /
            16384);
    });
    // Appreciate score best move
    table[player][best_move.0][best_move.1] += (current_depth * current_depth * MULTIPLIER) as i32
                                                - (table[player][best_move.0][best_move.1] * ((current_depth * current_depth * MULTIPLIER) as i32).abs() /
                                                    16384);
}
