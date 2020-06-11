extern crate rand;

use super::super::render::board;
use rand::Rng;

// Type of element in TT
#[derive(Copy, Clone, PartialEq)]
pub enum TypeOfEl {
    Lowerbound,
    Upperbound,
    Exact,
}

// Transposition table
#[derive(Clone, Copy)]
pub struct TT {
    pub is_valid: bool,
    // Zobrist key, to check for collision
    pub key: u64,
    // Values
    pub value: i64,
    pub r#type: TypeOfEl,
    pub depth: i8,
    pub r#move: Option<(usize, usize)>,
}

// Transposition table of at least 2^20 entries
// 2^22 here => 4 194 304 entries, from which we found
// the next prime : 4194319 (to avoid hash collision)
pub fn initialize_transposition_table() -> Vec<TT> {
    let initialized_struct = TT {
        key: 0,
        is_valid: false,
        value: 0,
        r#type: TypeOfEl::Exact,
        depth: 0,
        r#move: None,
    };
    vec![initialized_struct; 4194319]
}

pub fn retrieve_tt_from_hash(tt: &Vec<TT>, zhash: &u64) -> TT {
    tt[(*zhash % tt.len() as u64) as usize]
}

pub fn store_tt_entry(
    tt: &mut Vec<TT>,
    zhash: &mut u64,
    tte: TT
) -> () {
    let len = tt.len();
    tt[(*zhash % len as u64) as usize] = tte;
}

// Zobrist hash
pub const ZPIECES: [usize; 2] = [0, 1]; // 0 is black_pawn, 1 is white_pawn

// Initialize the first zobrist hash
// We initialize a 3D array of 19x19 containing for each cell
// An array of uniform random f64 number (2, one for each piece)
pub fn init_zboard() -> [[[u64; 2]; board::SIZE_BOARD]; board::SIZE_BOARD] {
    let mut table = [[[0_u64; 2]; board::SIZE_BOARD]; board::SIZE_BOARD];
    let mut rng = rand::thread_rng();

    for line in 0..board::SIZE_BOARD {
        for col in 0..board::SIZE_BOARD {
            for i in 0..2 {
                // Fill it with a uniformly generated f64 to avoid collision
                table[line][col][i] = rng.gen::<u64>();
            }
        }
    }
    table
}

// Function that initializes the zhash as a u64 accordingly to the current board's state
pub fn board_to_zhash(
    board: &[[Option<bool>; board::SIZE_BOARD]; board::SIZE_BOARD],
    ztable: &[[[u64; 2]; board::SIZE_BOARD]; board::SIZE_BOARD]
) -> u64 {
    let mut hash: u64 = 0;

    for line in 0..board::SIZE_BOARD {
        for col in 0..board::SIZE_BOARD {
            match board[line][col] {
                None => (),
                Some(true) => hash ^= ztable[line][col][ZPIECES[1]],
                Some(false) => hash ^= ztable[line][col][ZPIECES[0]],
            }
        }
    }
    hash
}
