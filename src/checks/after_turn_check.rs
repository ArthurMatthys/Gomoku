use super::super::model::game;

// Recursive function that recurses in the direction specified
// and counts the number of pawns of the same color
fn explore(
    board: &[Option<bool>; 361],
    direction: isize,
    index: &isize,
    type_of_index: Option<bool>,
    counter: usize,
) -> usize {
    if *index >= 0 && *index <= 361 && board[*index as usize] == type_of_index {
        explore(
            board,
            direction,
            &(*index + direction),
            type_of_index,
            counter + 1,
        )
    } else {
        counter
    }
}

// Macro that expands the if statement for more clarity
// It calls the explore function 2 times with the $direction and it's opposite
// And checks wether there are 5 or more pawn of the same color
//
// Ex:   explore($board, 20, $index_lpiece, $board[*$index_lpiece as usize], 0)
//       explore($board, -20, $index_lpiece, $board[*$index_lpiece as usize], 0)
macro_rules! check_winner {
    ($board:expr, $direction:expr, $index_lpiece:expr) => {
        explore(
            $board,
            $direction,
            $index_lpiece,
            $board[*$index_lpiece as usize],
            0,
        ) + explore(
            $board,
            -$direction,
            $index_lpiece,
            $board[*$index_lpiece as usize],
            0,
        ) - 1
            >= 5
    };
}

// Function that checks if a winner has been found
pub fn check_winner(game: &game::Game) -> bool {
    let board = game.board;
    if let Some(index_lpiece) = game.history.last() {
        // (1,1) = + 1 + 19
        if check_winner!(&board, 20, &(*index_lpiece as isize)) {
            true
        } else if check_winner!(&board, 19, &(*index_lpiece as isize)) {
            true
        } else if check_winner!(&board, 18, &(*index_lpiece as isize)) {
            true
        } else if check_winner!(&board, 1, &(*index_lpiece as isize)) {
            true
        } else {
            false
        }
    } else {
        false
    }
}
