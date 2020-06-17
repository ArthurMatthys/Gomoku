use super::super::checks::after_turn_check::DIRECTIONS;
use super::super::checks::capture::DIRS;
use super::super::checks::double_three::check_double_three_hint;
use super::super::render::board::SIZE_BOARD;

// use super::heuristic;
// use super::zobrist;


const AVRG_MAX_MULTIPLE_THREATS: usize = 3;
const MAX_MULTIPLE_DEFENSE_MOVES: usize = 4;


#[derive(Copy, Clone, PartialEq)]
enum TypeOfThreat {
    NONE,
    THREE_O,
    FOUR_O,
    FOUR_SO,
    FIVE_TAKE,
    FIVE,
    TAKE
}

// struct Threat {
//     x: u8,
//     y: u8,

// }

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

// (
//     Vec<(usize, usize)>,
//     Vec<((usize, usize), Vec<(usize, usize)>)>,
// )

pub fn threat_search_space(
    board: &mut [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    score_board: &mut [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD],
    actual_player: Option<bool>,
    actual_take: &mut isize,
) -> () {
    // 1. Initialize datastructures storing ready to be checked positions as well as threats
    let mut record: [[[bool; 4]; SIZE_BOARD]; SIZE_BOARD] = initialize_record(board, score_board, actual_player);
    
    // let tot = vec![vec![0_u8]];
    // 1.2. Initialize Threat board -> Vec containing with_capacity data (3 avrg max_possible threats per position) | (4 max defensive) 
    // DATASTRUCTURE NOT EASY WITH Rust memory management
    // [Vec<Vec<T>>; 19] <== Impossible, or not really flexible, need Copy Trait -> implies a copy for each modification
    // ===> EXTREMELY COSTLY IN RUST

    let mut hell: Vec<(TypeOfThreat, Vec<(usize,usize)>)> = (0..AVRG_MAX_MULTIPLE_THREATS).map(|_| (TypeOfThreat::NONE, Vec::with_capacity(MAX_MULTIPLE_DEFENSE_MOVES))).collect();
    
    let store = [[&mut hell;19]; 19];
    let mut threat_board: [[Vec<(TypeOfThreat, Vec<(usize,usize)>)>; SIZE_BOARD]; SIZE_BOARD] = (0..SIZE_BOARD).map(|_| (0..SIZE_BOARD).map(|_| hell).collect()).collect();
    // [[vec; SIZE_BOARD]; SIZE_BOARD];



    // 2. Parse board for actual_player's pawns
    for line in 0..SIZE_BOARD {
        for col in 0..SIZE_BOARD {
            if board[line][col] == actual_player {
                // 2. For a given position, in a given direction, if not empty, check if an extremity can be used
                // 3. If that's the case, find the corresponding response
                // 4. Store the interesting values inside the datastructure
            }
        }
    }

    // 5. Dispatch values inside the constructed datastructure in (1)

    // [[Vec<(enum, Vec<(usize,usize)>)>; SIZE_BOARD]; SIZE_BOARD]

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