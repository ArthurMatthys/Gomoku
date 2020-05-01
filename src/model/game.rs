extern crate rand;
extern crate sdl2;

use sdl2::render::Canvas;

use super::super::checks::after_turn_check;
use super::super::checks::capture;

use super::super::render::board;

use super::player;

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
            _ => None,
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

    fn clear_board(&mut self) -> () {
        if let Some(index) = self.history.pop() {
            self.board[index] = None;
            self.history.pop();
            self.has_changed = true;
        }
    }

    pub fn add_capture(&mut self) {
        match self.player_turn {
            0 => self.players.0.nb_of_catch += 1,
            1 => self.players.1.nb_of_catch += 1,
            _ => unreachable!(),
        }
    }

    fn clear_board_index(&mut self, (x,y):(isize, isize)) -> () {
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

    pub fn get_actual_player_mutable(&mut self) -> &mut player::Player {
        match self.player_turn {
            0 => &mut (self.players.0),
            1 => &mut (self.players.1),
            _ => unreachable!(),
        }
    }

    //    pub fn get_player_canvas(&self) -> &(&Canvas<sdl2::video::Window>, &player::Player) {
    //        &(self.canvas, self.get_actual_player())
    //    }

    pub fn change_board_from_input(&mut self, x: i32, y: i32) {
        let index: usize = (x * board::SIZE_BOARD as i32 + y) as usize;
        if index >= 361 {
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
        let index: usize = ((x / board::SQUARE_SIZE as i32) * board::SIZE_BOARD as i32
            + y / board::SQUARE_SIZE as i32) as usize;
        if index >= 361 {
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
        format!("nb of take : {}", self.players.0.nb_of_catch)
    }

    pub fn get_player2(&self) -> &str {
        match self.players.1.player_type {
            player::TypeOfPlayer::Human => "Player 2 : Human",
            player::TypeOfPlayer::Robot => "Player 2 : IA",
            player::TypeOfPlayer::Unset => unreachable!(),
        }
    }

    pub fn get_player2_take(&self) -> String {
        format!("nb of take : {}", self.players.1.nb_of_catch)
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
