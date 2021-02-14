use super::super::model::board::Board;
use super::super::model::score_board::ScoreBoard;
use std::sync::mpsc::{ channel, Sender, Receiver };
use std::time;

#[derive(Clone, Copy)]
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
    pub start_time: time::Instant,
    pub f: i64,
}

pub struct ThreadPool {
    pub pool:rayon::ThreadPool,
    pub tx: Sender<bool>,
    pub rx: Receiver<bool>,
}

impl ThreadPool {
    pub fn new() -> ThreadPool {
        let (tx,rx):(Sender<bool>,Receiver<bool>) = channel::<bool>();
        ThreadPool {
            pool: rayon::ThreadPoolBuilder::new()
                                            .num_threads(3)
                                            .build()
                                            .unwrap(),
            tx: tx,
            rx: rx,
        }
    }

    pub fn dumb_send(&self) -> () {
        for _ in 0..3 {
            self.tx.send(true).unwrap();
        };
    }

    pub fn update(&mut self) -> () {
        let (tx,rx):(Sender<bool>,Receiver<bool>) = channel::<bool>();
        self.tx = tx;
        self.rx = rx;
    }

    pub fn wait_threads(&self) -> () {
        println!("WAIT_THREADS");
        for _ in 0..3 {
            let _ = self.rx.recv().unwrap();
        }
    }
}