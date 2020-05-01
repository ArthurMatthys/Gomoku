use super::super::model::game;

pub const DIRS: [isize; 8] = [20, 19, 18, 1, -1, -18, -19, -20];


// Recursive function that recurses in the direction specified
// and counts the number of pawns of the same color
fn explore_capture(
    board: &[Option<bool>; 361],
    direction: isize,
    index: &isize,
    type_of_index: bool,
    counter: usize,
) -> Option<(isize, isize)> {
    if *index >= 0 && *index < 361 && counter < 2 && board[*index as usize] == Some(!type_of_index) {
        explore_capture(
            board,
            direction,
            &(*index + direction),
            type_of_index,
            counter + 1,
        )
    } else if *index >= 0 && *index < 361 && counter == 2 && board[*index as usize] == Some(type_of_index) {
        Some((index - direction, index - direction * 2))
    } else { None }
}



// Function that checks if a winner has been found
pub fn check_capture(game:&mut game::Game) -> Option<Vec<(isize, isize)>> {
    // let board = game.board;
    if let Some(index_lpiece) = game.history.last() {
        // I collect the type of the last piece addeded
        if let Some (piece) = game.board[*index_lpiece] {
            // Retrieves the map of true or false
           Some(DIRS.iter().filter_map(|&x| explore_capture(&game.board, x,
                &(*index_lpiece as isize + x), piece, 0)).collect::<Vec<(isize, isize)>>())
        } else { None }
    } else { None }
}
