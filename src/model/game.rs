extern crate rand;
extern crate sdl2;

use sdl2::render::Canvas;

use std::time::Duration;

use super::super::checks::after_turn_check;
use super::super::checks::capture;
use super::super::checks::valid_pos;
use super::super::checks::double_three;

use super::super::render::board;

use super::player;
use super::point;

const FORBIDDEN_PRO: [usize; 25] = [
    140, 141, 142, 143, 144, 159, 160, 161, 162, 163, 178, 179, 180, 181, 182, 197, 198, 199, 200,
    201, 216, 217, 218, 219, 220,
];
const FORBIDDEN_LONGPRO: [usize; 49] = [
    120, 121, 122, 123, 124, 125, 126, 139, 140, 141, 142, 143, 144, 145, 158, 159, 160, 161, 162,
    163, 164, 177, 178, 179, 180, 181, 182, 183, 196, 197, 198, 199, 200, 201, 202, 215, 216, 217,
    218, 219, 220, 221, 234, 235, 236, 237, 238, 239, 240,
];

macro_rules! string_of_index {
    ($e:expr) => {{
        let line: char = std::char::from_u32('A' as u32 + ($e / board::SIZE_BOARD) as u32)
            .expect("Could not convert number to char");
        let col = $e % board::SIZE_BOARD;
        format!("{}{}", line, col)
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
    pub history: Vec<usize>,
    pub history_capture: Vec<(usize, (usize, usize))>,

    pub board: [Option<bool>; 361],
    pub forbidden: Vec<usize>,
    pub capture: Vec<usize>,

    pub type_of_party: TypeOfParty,
    pub has_changed: bool,
    pub result: bool,
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
                board: [None; 361],
                type_of_party: type_of_party,
                has_changed: false,
                history: Vec::new(),
                history_capture: Vec::new(),
                result: false,
                forbidden: vec![],
                capture: vec![],
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
    //Modify board
    pub fn change_board_from_input(&mut self, index: usize) {
        if !valid_pos::valid_pos(self, index) {
            return;
        }
        match self.board[index] {
            Some(_) => (),
            None => {
                self.change_board_value(index);
                self.next_player()
            }
        }
    }

    fn change_board_value(&mut self, index: usize) -> () {
        self.board[index] = self.player_to_pawn();
        self.history.push(index);
        self.result = after_turn_check::check_winner(&self);
        if let Some(ret) = capture::check_capture(self) {
            ret.iter().for_each(|&x| self.clear_board_index(x, index));
        }
        self.set_changed();
    }

    pub fn change_board_from_click(&mut self, x: i32, y: i32) {
        let index: usize =
            (x as usize / board::SQUARE_SIZE) * board::SIZE_BOARD + y as usize / board::SQUARE_SIZE;
        if !valid_pos::valid_pos(self, index) {
            return;
        }
        match self.board[index] {
            Some(_) => (),
            None => {
                self.change_board_value(index);
                self.next_player()
            }
        }
    }

    pub fn change_board_value_hint(&mut self, index: usize) -> () {
        self.board[index] = self.player_to_pawn();
        self.history.push(index);
        self.next_player();
    }

    pub fn clear_last_move(&mut self) -> () {
        let mut new_history = vec![];
        if let Some(index) = self.history.pop() {
            let mut nbr = 0;
            self.board[index] = None;
            for (x, (y, z)) in self.history_capture.iter() {
                if *x == index {
                    self.board[*y] = self.player_to_pawn();
                    self.board[*z] = self.player_to_pawn();
                    nbr += 1;
                } else {
                    new_history.push((*x, (*y, *z)));
                }
            }
            for _ in 0..nbr {
                self.minus_capture();
            }
            self.history_capture = new_history;
            self.set_changed();
            self.next_player();
        }
    }

    fn add_history_capture(&mut self, (x, y): (isize, isize), index: usize) -> () {
        self.history_capture.push((index, (x as usize, y as usize)));
    }

    fn clear_board_index(&mut self, (x, y): (isize, isize), index: usize) -> () {
        self.add_history_capture((x, y), index);
        self.board[x as usize] = None;
        self.board[y as usize] = None;
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

    fn add_impossible_index(&mut self, point: usize) -> () {
        self.forbidden.push(point);
    }

    fn add_impossible_vec_index(&mut self, points: Vec<usize>) -> () {
        points
            .iter()
            .for_each(|&point| self.add_impossible_index(point));
    }

    fn add_capture_index(&mut self, point: usize) -> () {
        self.capture.push(point);
    }

    fn add_capture_vec_index(&mut self, points: Vec<usize>) -> () {
        points
            .iter()
            .for_each(|&point| self.add_capture_index(point));
    }

    pub fn set_forbidden_pos(&mut self) -> () {
        self.clear_forbidden();
        match self.type_of_party {
            TypeOfParty::Pro => match self.history.len() {
                0 => self.add_impossible_vec_index(valid_pos::all_except(vec![180])),
                2 => self.add_impossible_vec_index(FORBIDDEN_PRO.to_vec()),
                _ => ()
            },
            TypeOfParty::Longpro => match self.history.len() {
                0 => self.add_impossible_vec_index(valid_pos::all_except(vec![180])),
                2 => self.add_impossible_vec_index(FORBIDDEN_LONGPRO.to_vec()),
                _ => ()
            },
            TypeOfParty::Standard => ()
        }
        let double_threes = double_three::forbidden_indexes(&self);
        match double_threes {
            Some(x) => self.add_impossible_vec_index(x),
            None => (),
        }
    }

    pub fn set_capture_pos(&mut self) -> () {
        self.clear_capture();
        let capture = capture::find_capture(self);
        self.add_capture_vec_index(capture);
        self.has_changed = true;
    }

    pub fn is_forbidden_from_index(&self, index: usize) -> bool {
        self.forbidden.iter().any(|&point| point == index)
    }

    pub fn is_forbidden_from_coord(&self, x: usize, y: usize) -> bool {
        self.forbidden
            .iter()
            .any(|&point| point == point::index_of_coord(x, y))
    }

    pub fn is_capture_from_coord(&self, x: usize, y: usize) -> bool {
        self.capture
            .iter()
            .any(|&point| point == point::index_of_coord(x, y))
    }

    pub fn set_changed(&mut self) -> () {
        self.set_forbidden_pos();
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
            .map(|(_, e)| string_of_index!(e))
            .collect::<Vec<String>>();
        let white_history: Vec<String> = self
            .history
            .iter()
            .enumerate()
            .filter(|&(i, _)| i % 2 == 1)
            .map(|(_, e)| string_of_index!(e))
            .collect::<Vec<String>>();
        (black_history, white_history)
    }
}
