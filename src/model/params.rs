use super::super::model::board::Board;
use super::super::model::score_board::ScoreBoard;
use std::time;

// #[derive(Clone, Copy)]
pub struct ParamsIA {
    pub board: Board,
    pub score_board: ScoreBoard,
    pub zhash: u64,
    pub current_depth: i8,
    pub actual: Option<bool>,
    pub actual_catch: isize,
    pub opp_catch: isize,
    pub alpha: i64,
    pub beta: i64,
    pub color: i8,
    pub depth_max: i8,
    pub counter_tree: u64,
    pub start_time: time::Instant
}