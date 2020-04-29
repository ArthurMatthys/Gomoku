extern crate rand;
use super::point;
use rand::Rng;

// Enum representing the different types of parties
//#[derive(Copy, Clone)]
pub enum TypeOfPlayer {
    Unset,
    Human,
    Robot,
}

// Struct representing the infos of the current Game
// Board is 361 cases long -> 19 x 19 Board
//#[derive(Copy, Clone)]
pub struct Player {
    player_type: TypeOfPlayer,
    pub nb_of_catch: isize,
    pub bool_type: Option<bool>,
    pub forbidden: Vec<point::Point>,
    name: &'static str,
}

impl Player {
    // Getter of the Game Instance
    pub fn new() -> Player {
        Player {
            player_type: TypeOfPlayer::Unset,
            nb_of_catch: 0,
            bool_type: None,
            forbidden: vec![],
            name: "",
        }
    }

    // Setter of the Game Instance
    pub fn set_infos(
        &mut self,
        player_type: TypeOfPlayer,
        bool_type: Option<bool>,
        name: &'static str,
    ) -> () {
        self.player_type = player_type;
        self.bool_type = bool_type;
        self.name = name
    }

    pub fn get_forbidden(&self) -> &Vec<point::Point> {
        &self.forbidden
    }

    pub fn get_mutable_forbidden(&mut self) -> &mut Vec<point::Point> {
        &mut self.forbidden
    }

    pub fn set_forbidden(&mut self, forbidden: Vec<point::Point>) -> () {
        self.forbidden = forbidden
    }

    pub fn test_forbidden(&mut self) -> () {
        let mut rng = rand::thread_rng();
        let mut new_forbid = vec![];
        for _ in 0..20 {
            new_forbid.push(point::Point::new(
                rng.gen_range(0, 19),
                rng.gen_range(0, 19),
            ));
        }
        self.set_forbidden(new_forbid);
    }
}

// Aim of the function :
// Function that initializes the 2 players regarding the Gameplay chosen
pub fn initialize_players(nb_player: usize) -> (Player, Player) {
    let mut player1: Player = Player::new();
    let mut player2: Player = Player::new();
    match nb_player {
        0 => {
            player1.set_infos(TypeOfPlayer::Robot, Some(true), "Robot1");
            player2.set_infos(TypeOfPlayer::Robot, Some(false), "Robot2");
        }
        1 => {
            player1.set_infos(TypeOfPlayer::Human, Some(true), "Human1");
            player2.set_infos(TypeOfPlayer::Robot, Some(false), "Robot1");
        }
        2 => {
            player1.set_infos(TypeOfPlayer::Human, Some(true), "Human1");
            player2.set_infos(TypeOfPlayer::Human, Some(false), "Human2");
        }
        _ => {
            player1.set_infos(TypeOfPlayer::Unset, Some(true), "Unset1");
            player2.set_infos(TypeOfPlayer::Unset, Some(false), "Unset2");
        }
    };
    let mut rng = rand::thread_rng();

    match rng.gen_range(0, 2) {
        0 => (player1, player2),
        1 => (player2, player1),
        _ => (player1, player2),
    }
}