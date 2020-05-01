extern crate rand;
extern crate sdl2;

use sdl2::render::Canvas;

use std::time::Duration;

use super::super::checks::after_turn_check;
use super::super::checks::capture;
use super::super::checks::valid_pos;

use super::super::render::board;

use super::player;
use super::point;

const COMPULSORY: [usize; 360] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
    74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97,
    98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
    117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135,
    136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154,
    155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173,
    174, 175, 176, 177, 178, 179, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192, 193,
    194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211, 212,
    213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231,
    232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250,
    251, 252, 253, 254, 255, 256, 257, 258, 259, 260, 261, 262, 263, 264, 265, 266, 267, 268, 269,
    270, 271, 272, 273, 274, 275, 276, 277, 278, 279, 280, 281, 282, 283, 284, 285, 286, 287, 288,
    289, 290, 291, 292, 293, 294, 295, 296, 297, 298, 299, 300, 301, 302, 303, 304, 305, 306, 307,
    308, 309, 310, 311, 312, 313, 314, 315, 316, 317, 318, 319, 320, 321, 322, 323, 324, 325, 326,
    327, 328, 329, 330, 331, 332, 333, 334, 335, 336, 337, 338, 339, 340, 341, 342, 343, 344, 345,
    346, 347, 348, 349, 350, 351, 352, 353, 354, 355, 356, 357, 358, 359, 360,
];
const FORBIDDEN_PRO: [usize; 25] = [
    140, 141, 142, 143, 144, 159, 160, 161, 162, 163, 178, 179, 180, 181, 182, 197, 198, 199, 200,
    201, 216, 217, 218, 219, 220,
];
const FORBIDDEN_LONGPRO: [usize; 49] = [
    120, 121, 122, 123, 124, 125, 126, 139, 140, 141, 142, 143, 144, 145, 158, 159, 160, 161, 162,
    163, 164, 177, 178, 179, 180, 181, 182, 183, 196, 197, 198, 199, 200, 201, 202, 215, 216, 217,
    218, 219, 220, 221, 234, 235, 236, 237, 238, 239, 240,
];
//use super::point;
//use rand::Rng;

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
    pub board: [Option<bool>; 361],
    pub history: Vec<usize>,
    pub has_changed: bool,
    pub forbidden: Vec<point::Point>,

    pub type_of_party: TypeOfParty,
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
                has_changed: true,
                history: Vec::new(),
                result: false,
                forbidden: vec![],
            },
            events,
        ))
    }

    fn next_player(&mut self) -> () {
        self.player_turn = (self.player_turn + 1) % 2;
    }

    fn player_to_pawn(&self) -> Option<bool> {
        match self.player_turn {
            0 => Some(false),
            1 => Some(true),
            _ => unreachable!(),
        }
    }

    fn change_board_value(&mut self, index: usize) -> () {
        self.board[index] = self.player_to_pawn();
        self.history.push(index);
        self.result = after_turn_check::check_winner(&self);
        if let Some(ret) = capture::check_capture(self) {
            ret.iter().for_each(|&x| self.clear_board_index(x));
        }
        self.has_changed = true;
    }

    pub fn clear_board(&mut self) -> () {
        if let Some(index) = self.history.pop() {
            self.board[index] = None;
            self.has_changed = true;
            self.next_player();
        }
    }

    pub fn add_capture(&mut self) {
        match self.player_turn {
            0 => self.players.0.nb_of_catch += 1,
            1 => self.players.1.nb_of_catch += 1,
            _ => unreachable!(),
        }
    }

    fn clear_board_index(&mut self, (x, y): (isize, isize)) -> () {
        self.board[x as usize] = None;
        self.board[y as usize] = None;
        self.add_capture();
        self.has_changed = true;
    }

    pub fn get_actual_player(&self) -> &player::Player {
        match self.player_turn {
            0 => &self.players.0,
            1 => &self.players.1,
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

    //    pub fn get_actual_player_mutable(&mut self) -> &mut player::Player {
    //        match self.player_turn {
    //            0 => &mut (self.players.0),
    //            1 => &mut (self.players.1),
    //            _ => unreachable!(),
    //        }
    //    }

    pub fn set_player_time(&mut self, time: Duration) -> () {
        match self.player_turn {
            0 => self.players.0.set_time(time),
            1 => self.players.1.set_time(time),
            _ => unreachable!(),
        }
    }

    //    pub fn get_player_canvas(&self) -> &(&Canvas<sdl2::video::Window>, &player::Player) {
    //        &(self.canvas, self.get_actual_player())
    //    }

    pub fn change_board_from_input(&mut self, point: &point::Point) {
        let index: usize = (point.x * board::SIZE_BOARD + point.y) as usize;
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

    fn clear_impossible(&mut self) -> () {
        self.forbidden = vec![];
    }

    fn add_impossible(&mut self, point: point::Point) -> () {
        self.forbidden.push(point);
    }

    pub fn set_impossible_pos(&mut self) -> () {
        self.clear_impossible();
        match self.type_of_party {
            TypeOfParty::Pro => match self.history.len() {
                0 => COMPULSORY
                    .iter()
                    .for_each(|i| self.add_impossible(point::point_of_index(i))),
                2 => FORBIDDEN_PRO
                    .iter()
                    .for_each(|i| self.add_impossible(point::point_of_index(i))),
                _ => (),
            },
            TypeOfParty::Longpro => match self.history.len() {
                0 => COMPULSORY
                    .iter()
                    .for_each(|i| self.add_impossible(point::point_of_index(i))),
                2 => FORBIDDEN_LONGPRO
                    .iter()
                    .for_each(|i| self.add_impossible(point::point_of_index(i))),
                _ => (),
            },
            TypeOfParty::Standard => {}
        }
    }
}
