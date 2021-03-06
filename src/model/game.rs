extern crate rand;
extern crate sdl2;

use sdl2::render::Canvas;

use super::super::checks::after_turn_check;
use super::super::checks::capture;
use super::super::checks::double_three;
use super::super::checks::valid_pos;
use super::super::ia::heuristic;
use std::time::Duration;

use super::super::render::board;

use super::player;

const FORBIDDEN_PRO: [(usize, usize); 25] = [
    (7, 7),
    (8, 7),
    (9, 7),
    (10, 7),
    (11, 7),
    (7, 8),
    (8, 8),
    (9, 8),
    (10, 8),
    (11, 8),
    (7, 9),
    (8, 9),
    (9, 9),
    (10, 9),
    (11, 9),
    (7, 10),
    (8, 10),
    (9, 10),
    (10, 10),
    (11, 10),
    (7, 11),
    (8, 11),
    (9, 11),
    (10, 11),
    (11, 11),
];

const FORBIDDEN_LONGPRO: [(usize, usize); 49] = [
    (6, 6),
    (7, 6),
    (8, 6),
    (9, 6),
    (10, 6),
    (11, 6),
    (12, 6),
    (6, 7),
    (7, 7),
    (8, 7),
    (9, 7),
    (10, 7),
    (11, 7),
    (12, 7),
    (6, 8),
    (7, 8),
    (8, 8),
    (9, 8),
    (10, 8),
    (11, 8),
    (12, 8),
    (6, 9),
    (7, 9),
    (8, 9),
    (9, 9),
    (10, 9),
    (11, 9),
    (12, 9),
    (6, 10),
    (7, 10),
    (8, 10),
    (9, 10),
    (10, 10),
    (11, 10),
    (12, 10),
    (6, 11),
    (7, 11),
    (8, 11),
    (9, 11),
    (10, 11),
    (11, 11),
    (12, 11),
    (6, 12),
    (7, 12),
    (8, 12),
    (9, 12),
    (10, 12),
    (11, 12),
    (12, 12),
];

macro_rules! string_of_index {
    ($line:expr, $col:expr) => {{
        let col: char = std::char::from_u32('A' as u32 + *$col as u32)
            .expect("Could not convert number to char");
        let line = *$line;
        format!("{}{}", col, line)
    }};
}

// TYPE OF PARTY
pub enum TypeOfParty {
    Standard,
    Pro,
    Longpro,
}

pub struct Game {
    // DESIGN
    pub canvas: Canvas<sdl2::video::Window>,

    // GAME
    player_turn: i32,
    pub players: (player::Player, player::Player),
    pub history: Vec<(usize, usize)>,
    pub history_capture: Vec<((usize, usize), ((usize, usize), (usize, usize)))>,

    pub board: [[Option<bool>; board::SIZE_BOARD]; board::SIZE_BOARD],
    pub forbidden: Vec<(usize, usize)>,
    pub capture: Vec<(usize, usize)>,
    pub best_move: Vec<(usize, usize)>,

    pub type_of_party: TypeOfParty,
    pub has_changed: bool,
    pub result: Option<bool>,
    pub instant_win: bool,
    pub winner: Option<bool>,
    pub firstguess: (i64, i64),
}

impl Game {
    pub fn new(
        title: &'static str,
        width: u32,
        height: u32,
        nb_of_player: usize,
        type_of_party: TypeOfParty,
    ) -> Result<(Game, sdl2::EventPump), String> {
        // Initialize SDL2
        let sdl_context = sdl2::init().expect("SDL initialization failed");
        let video = sdl_context
            .video()
            .expect("Couldn't get SDL video subsystem");

        // Create the window
        let window = video
            .window(title, width, height)
            .position_centered()
            .opengl()
            .build()
            .expect("Failed to create window");

        let events = sdl_context.event_pump()?;

        let canvas = window
            .into_canvas()
            .accelerated()
            .build()
            .expect("Failed to convert window into canvas");

        Ok((
            Game {
                canvas: canvas,
                players: player::initialize_players(nb_of_player),
                player_turn: 0,
                board: [[None; 19]; 19],
                type_of_party: type_of_party,
                has_changed: false,
                history: Vec::new(),
                history_capture: Vec::new(),
                result: None,
                forbidden: vec![],
                capture: vec![],
                best_move: vec![],
                instant_win: false,
                winner: None,
                firstguess: (0, 0),
            },
            events,
        ))
    }

    pub fn get_actual_player(&self) -> &player::Player {
        match self.player_turn {
            0 => &self.players.0,
            1 => &self.players.1,
            _ => unreachable!(),
        }
    }

    pub fn get_opponent(&self) -> &player::Player {
        match self.player_turn {
            0 => &self.players.1,
            1 => &self.players.0,
            _ => unreachable!(),
        }
    }

    fn next_player(&mut self) -> () {
        self.player_turn = (self.player_turn + 1) % 2;
    }

    fn add_capture(&mut self) {
        match self.player_turn {
            0 => self.players.0.nb_of_catch += 1,
            1 => self.players.1.nb_of_catch += 1,
            _ => unreachable!(),
        }
    }

    fn minus_capture(&mut self) {
        match self.player_turn {
            1 => self.players.0.nb_of_catch -= 1,
            0 => self.players.1.nb_of_catch -= 1,
            _ => unreachable!(),
        }
    }

    pub fn player_to_pawn(&self) -> Option<bool> {
        match self.player_turn {
            0 => Some(false),
            1 => Some(true),
            _ => unreachable!(),
        }
    }

    pub fn actual_player_is_ai(&self) -> Option<bool> {
        let player = self.get_actual_player();
        match player.player_type {
            player::TypeOfPlayer::Unset => None,
            player::TypeOfPlayer::Human => Some(false),
            player::TypeOfPlayer::Robot => Some(true),
        }
    }

    pub fn set_player_time(&mut self, time: Duration) -> () {
        match self.player_turn {
            0 => self.players.0.set_time(time),
            1 => self.players.1.set_time(time),
            _ => unreachable!(),
        }
    }
}

impl Game {
    //Modify board when user click
    pub fn change_board_from_input(&mut self, line: usize, col: usize) {
        if !valid_pos::valid_pos(self, line, col) {
            println!("here also you loose");
            return;
        }
        match self.board[line][col] {
            Some(_) => println!("You loose"),
            None => {
                self.change_board_value(line, col);
                self.next_player()
            }
        }
    }

    fn change_board_value(&mut self, line: usize, col: usize) -> () {
        self.board[line][col] = self.player_to_pawn();
        self.history.push((line, col));
        if let Some(ret) = capture::check_capture(self) {
            ret.iter()
                .for_each(|&x| self.clear_board_index(x, line, col));
        }
        self.set_changed();
    }

    pub fn change_board_from_click(&mut self, x: i32, y: i32) {
        let new_x = x as usize / board::SQUARE_SIZE;
        let new_y = y as usize / board::SQUARE_SIZE;
        if new_x * new_y == 0 {
            return;
        }
        if !valid_pos::valid_pos(self, new_y - 1, new_x - 1) {
            return;
        }
        match self.board[new_y - 1][new_x - 1] {
            Some(_) => (),
            None => {
                self.change_board_value(new_y - 1, new_x - 1);
                self.next_player();
            }
        }
    }

    pub fn change_board_value_hint(&mut self, line: usize, col: usize) -> () {
        self.board[line][col] = self.player_to_pawn();
        self.history.push((line, col));
        self.next_player();
    }

    pub fn clear_last_move(&mut self) -> () {
        if let Some((line, col)) = self.history.pop() {
            let mut nbr = 0;
            self.board[line][col] = None;
            for (x, ((line_y, col_y), (line_z, col_z))) in self.history_capture.iter().rev() {
                if *x == (line, col) {
                    self.board[*line_y][*col_y] = self.player_to_pawn();
                    self.board[*line_z][*col_z] = self.player_to_pawn();
                    nbr += 1;
                } else {
                    break;
                }
            }
            for _ in 0..nbr {
                self.minus_capture();
            }
            let _ = self
                .history_capture
                .split_off(self.history_capture.len() - nbr);
            self.set_changed();
            self.next_player();
        }
    }

    fn add_history_capture(
        &mut self,
        ((line_x, col_x), (line_y, col_y)): ((isize, isize), (isize, isize)),
        line: usize,
        col: usize,
    ) -> () {
        self.history_capture.push((
            (line, col),
            (
                (line_x as usize, col_x as usize),
                (line_y as usize, col_y as usize),
            ),
        ));
    }

    fn clear_board_index(
        &mut self,
        ((line_x, col_x), (line_y, col_y)): ((isize, isize), (isize, isize)),
        line: usize,
        col: usize,
    ) -> () {
        self.add_history_capture(((line_x, col_x), (line_y, col_y)), line, col);
        self.board[line_x as usize][col_x as usize] = None;
        self.board[line_y as usize][col_y as usize] = None;
        self.add_capture();
        self.set_changed();
    }
}

impl Game {
    //render board
    fn clear_forbidden(&mut self) -> () {
        self.forbidden = vec![];
    }

    fn clear_capture(&mut self) -> () {
        self.capture = vec![];
    }

    pub fn clear_best_move(&mut self) -> () {
        self.best_move = vec![];
    }

    fn add_impossible_index(&mut self, point: (usize, usize)) -> () {
        self.forbidden.push(point);
    }

    fn add_impossible_vec_index(&mut self, points: Vec<(usize, usize)>) -> () {
        points
            .iter()
            .for_each(|&point| self.add_impossible_index(point));
    }

    fn add_capture_index(&mut self, point: (usize, usize)) -> () {
        self.capture.push(point);
    }

    fn add_best_move_index(&mut self, point: (usize, usize)) -> () {
        self.best_move.push(point);
    }

    fn add_capture_vec_index(&mut self, points: Vec<(usize, usize)>) -> () {
        points
            .iter()
            .for_each(|&point| self.add_capture_index(point));
    }

    fn add_best_move_vec_index(&mut self, points: Vec<(usize, usize)>) -> () {
        points
            .iter()
            .for_each(|&point| self.add_best_move_index(point));
    }

    pub fn set_forbidden_pos(&mut self) -> () {
        self.clear_forbidden();
        match self.type_of_party {
            TypeOfParty::Pro => match self.history.len() {
                0 => self.add_impossible_vec_index(valid_pos::all_except(vec![(9, 9)])),
                2 => self.add_impossible_vec_index(FORBIDDEN_PRO.to_vec()),
                _ => (),
            },
            TypeOfParty::Longpro => match self.history.len() {
                0 => self.add_impossible_vec_index(valid_pos::all_except(vec![(9, 9)])),
                2 => self.add_impossible_vec_index(FORBIDDEN_LONGPRO.to_vec()),
                _ => (),
            },
            TypeOfParty::Standard => (),
        }
        let double_threes = double_three::check_double_three(self);
        self.add_impossible_vec_index(double_threes);
    }

    pub fn set_capture_pos(&mut self) -> () {
        self.clear_capture();
        let capture = capture::find_capture(self);
        self.add_capture_vec_index(capture);
        self.has_changed = true;
    }

    pub fn set_best_move(&mut self, x: usize, y: usize) -> () {
        self.clear_best_move();
        let mut new_vec = Vec::with_capacity(2);
        new_vec.push((x, y));
        self.add_best_move_vec_index(new_vec);
        self.has_changed = true;
    }

    pub fn is_forbidden_from_index(&self, line: usize, col: usize) -> bool {
        self.forbidden.iter().any(|&point| point == (line, col))
    }

    pub fn is_forbidden_from_coord(&self, x: usize, y: usize) -> bool {
        self.forbidden.iter().any(|&point| point == (x, y))
    }

    pub fn is_capture_from_coord(&self, x: usize, y: usize) -> bool {
        self.capture.iter().any(|&point| point == (x, y))
    }

    pub fn is_best_move_from_coord(&self, x: usize, y: usize) -> bool {
        self.best_move.iter().any(|&point| point == (x, y))
    }

    pub fn set_changed(&mut self) -> () {
        self.set_forbidden_pos();
        self.clear_best_move();
        self.clear_capture();
        self.has_changed = true;
    }
}

impl Game {
    //render score
    pub fn party_to_string(&self) -> &str {
        match self.type_of_party {
            TypeOfParty::Standard => "Party Type : Standard",
            TypeOfParty::Pro => "Party Type : Pro",
            TypeOfParty::Longpro => "Party Type : Long pro",
        }
    }

    pub fn get_player1(&self) -> &str {
        match self.players.0.player_type {
            player::TypeOfPlayer::Human => "Player 1 : Human",
            player::TypeOfPlayer::Robot => "Player 1 : IA",
            player::TypeOfPlayer::Unset => unreachable!(),
        }
    }

    pub fn get_player1_take(&self) -> String {
        format!("number of take : {}", self.players.0.nb_of_catch)
    }

    pub fn get_player2(&self) -> &str {
        match self.players.1.player_type {
            player::TypeOfPlayer::Human => "Player 2 : Human",
            player::TypeOfPlayer::Robot => "Player 2 : IA",
            player::TypeOfPlayer::Unset => unreachable!(),
        }
    }

    pub fn get_player2_take(&self) -> String {
        format!("number of take : {}", self.players.1.nb_of_catch)
    }

    pub fn get_player_turn_display(&self) -> &str {
        match self.player_turn {
            0 => "player1's turn",
            1 => "player2's turn",
            _ => unreachable!(),
        }
    }

    pub fn get_history(&self) -> (Vec<String>, Vec<String>) {
        let black_history: Vec<String> = self
            .history
            .iter()
            .enumerate()
            .filter(|&(i, _)| i % 2 == 0)
            .map(|(_, (line, col))| string_of_index!(col, line))
            .collect::<Vec<String>>();
        let white_history: Vec<String> = self
            .history
            .iter()
            .enumerate()
            .filter(|&(i, _)| i % 2 == 1)
            .map(|(_, (line, col))| string_of_index!(col, line))
            .collect::<Vec<String>>();
        (black_history, white_history)
    }
}

impl Game {
    pub fn check_win(&mut self) -> bool {
        if !self.has_changed {
            false
        } else {
            if self.players.0.nb_of_catch >= 5 || self.players.1.nb_of_catch >= 5 {
                self.result = None;
                self.instant_win = true;
                self.next_player();
                self.winner = self.player_to_pawn();
                true
            } else if let Some(winner) = self.result {
                if self.result != self.player_to_pawn() {
                    let score_board = heuristic::evaluate_board(&mut self.board.into());
                    for x in 0..19 {
                        for y in 0..19 {
                            if Some(!winner) == self.board[x][y] {
                                for dir in 0..4 {
                                    if score_board.get(x, y, dir).0 >= 5 {
                                        self.instant_win = true;
                                        self.winner = self.player_to_pawn();
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                    self.result = None;
                    self.check_win()
                } else {
                    false
                }
            } else if let Some(indexes) = after_turn_check::check_winner(self) {
                self.result = self.player_to_pawn();
                if capture::can_capture_vec(self, indexes) {
                    false
                } else {
                    let player = self.get_actual_player();
                    if player.nb_of_catch == 4 && capture::can_capture(self) {
                        false
                    } else {
                        self.result = None;
                        self.instant_win = true;
                        self.winner = match self.player_to_pawn() {
                            Some(a) => Some(!a),
                            _ => unreachable!(),
                        };
                        true
                    }
                }
            } else {
                false
            }
        }
    }
}
