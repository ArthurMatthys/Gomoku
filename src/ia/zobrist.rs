extern crate rand;

use rand::Rng;
use super::super::render::board;

// Type of element in TT
#[derive(Copy,Clone,PartialEq)]
pub enum TypeOfEl {
    Lowerbound,
    Upperbound,
    Exact,
    Empty,
}

// Type of element in TT
#[derive(Copy,Clone,PartialEq)]
pub enum Move {
    Leaf,
    Unitialized,
    Some((usize, usize)),
}

impl Move {
    pub fn unwrap_unsafe(&self) -> (usize,usize) {
        match self {
            Move::Some((i,j)) =>  (*i,*j),
            _ => unreachable!(),
        }
    }
}

// Transposition table
#[derive(Clone, Copy)]
pub struct TT {
    // Zobrist key, to check for collision
    pub key: u64,
    // Values
    pub value: i32,
    pub r#type: TypeOfEl,
    pub depth: i8,
    pub r#move: Move,
}

// Transposition table of at least 2^20 entries
// 2^22 here => 4 194 304 entries, from which we found
// the next prime : 4194319 (to avoid hash collision)
pub fn initialize_transposition_table() -> Vec<TT> {
    let initialized_struct = TT {
        key: 0,
        value: 0,
        r#type: TypeOfEl::Empty,
        depth: 0,
        r#move: Move::Unitialized,
    };
    vec![initialized_struct; 4194319]
}

pub fn retrieve_tt_from_hash(tt: &Vec<TT>, zhash: &u64) -> TT {
    tt[(*zhash % tt.len() as u64) as usize]
}

pub fn store_tt_entry(
    tt: &mut Vec<TT>,
    zhash: &mut u64,
    value: &i32,
    flag:TypeOfEl,
    depth:&mut i8,
    status: Move,
) -> () {
    let len = tt.len();
    tt[(*zhash % len as u64) as usize] = TT {
        key: *zhash,
        value: *value,
        r#type: flag,
        depth: *depth,
        r#move: status,
    }
}

// Zobrist hash
const ZPIECES: [usize; 2] = [0,1]; // 0 is black_pawn, 1 is white_pawn

// Initialize the first zobrist hash
// We initialize a 3D array of 19x19 containing for each cell
// An array of uniform random f64 number (2, one for each piece)
fn init_zboard() -> [[[u64; 2]; board::SIZE_BOARD]; board::SIZE_BOARD] {
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
pub fn board_to_zhash(board: &[[Option<bool>; board::SIZE_BOARD]; board::SIZE_BOARD]) -> ([[[u64; 2]; 19]; 19], u64) {
    let table = init_zboard();
    let mut hash: u64 = 0;

    for line in 0..board::SIZE_BOARD {    
        for col in 0..board::SIZE_BOARD {
            match board[line][col] {
                None => (),
                Some(true) => hash ^= table[line][col][ZPIECES[1]],
                Some(false) => hash ^= table[line][col][ZPIECES[0]],
            }
        }
    }
    (table, hash)
}

pub fn add_pawn_zhash(table:&[[[u64; 2]; 19]; 19], hash: &mut u64, (line, col, piece):(usize, usize, usize)) -> () {
    *hash ^= table[line][col][ZPIECES[piece]];
}

pub fn capture_zhash(
    table: &[[[u64; 2]; 19]; 19],
    hash: &mut u64,
    piece: usize,
    ((line1, col1),(line2, col2)):((isize, isize),(isize, isize))
) -> () {
    add_pawn_zhash(table, hash, (line1 as usize, col1 as usize, piece));
    add_pawn_zhash(table, hash, (line2 as usize, col2 as usize, piece));
}

