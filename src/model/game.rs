extern crate rand;
extern crate sdl2;

use super::player;
use super::point;
use rand::Rng;
use sdl2::render::Canvas;

pub const SIZE_BOARD: usize = 19;
const SQUARE_SIZE: usize = 51;
// TYPE OF PARTY
pub enum TypeOfParty {
    Standard,
    Pro,
    Longpro
}

pub struct Game {
    // DESIGN
    pub canvas: Canvas<sdl2::video::Window>,

    // GAME
    player_turn: i32,
    pub players: (player::Player, player::Player),
    pub board: [Option<bool>; 361],
    pub history: Vec<usize>,

    type_of_party: TypeOfParty,
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
                history: Vec::new(),
            },
            events,
        ))
    }

    fn next_player(&mut self) -> () {
        self.player_turn = (self.player_turn + 1) % 2;
    }

    fn player_to_pawn(&self) -> Option<bool> {
        match self.player_turn {
            0 => Some(true),
            1 => Some(false),
            _ => None,
        }
    }

    fn change_board_value(&mut self, index: usize) -> () {
        self.board[index] = self.player_to_pawn();
        self.history.push(index);
    }

    fn clear_board(&mut self) -> () {
        if let Some(index) = self.history.pop() {
            self.board[index] = None;
        }
    }

    pub fn get_actual_player(&self) -> &player::Player {
        match self.player_turn {
            0 => &self.players.0,
            1 => &self.players.1,
            _ => &self.players.1,
        }
    }

    pub fn get_actual_player_mutable(&mut self) -> &mut player::Player {
        match self.player_turn {
            0 => &mut (self.players.0),
            1 => &mut (self.players.1),
            _ => &mut (self.players.1),
        }
    }

    //    pub fn get_player_canvas(&self) -> &(&Canvas<sdl2::video::Window>, &player::Player) {
    //        &(self.canvas, self.get_actual_player())
    //    }

    pub fn change_board(&mut self, x: i32, y: i32) {
        let index: usize =
            ((x / SQUARE_SIZE as i32) * SIZE_BOARD as i32 + y / SQUARE_SIZE as i32) as usize;
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
}
