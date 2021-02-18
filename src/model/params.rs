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
    pub counter: u64,
}

pub static mut STOP_THREADS: bool = false;

pub fn reset_stop_thread() -> () {
    unsafe {
        STOP_THREADS = false;
    }
}

const LIMIT_DURATION: time::Duration = time::Duration::from_millis(495);

impl ParamsIA {
    pub fn check_timeout(&mut self) -> bool {
        if unsafe { STOP_THREADS } {
            return true
        }
        self.counter += 1;
        if self.counter >= 1000 {
            if time::Instant::now().duration_since(self.start_time) >= LIMIT_DURATION {
                unsafe { STOP_THREADS = true; }
                return true
            } else {
                self.counter = 0;
                return false
            }
        }
        false
    }
}


pub struct ThreadPool {
    pub pool:rayon::ThreadPool,
    pub counter: u64,
}

impl ThreadPool {
    pub fn new() -> ThreadPool {
        // let (tx,rx):(Sender<bool>,Receiver<bool>) = channel::<bool>();
        ThreadPool {
            pool: rayon::ThreadPoolBuilder::new()
                                            .num_threads(3)
                                            .build()
                                            .unwrap(),
            counter: 0,
        }
    }

    
    // pub fn dumb_send(&self) -> () {
    //     for _ in 0..3 {
    //         self.tx.send(true).unwrap();
    //     };
    // }

    // pub fn update(&mut self) -> () {
    //     let (tx,rx):(Sender<bool>,Receiver<bool>) = channel::<bool>();
    //     self.tx = tx;
    //     self.rx = rx;
    // }

    // pub fn wait_threads(&self) -> () {
    //     println!("WAIT_THREADS");
    //     for _ in 0..3 {
    //         let _ = self.rx.recv().unwrap();
    //     }
    // }
}