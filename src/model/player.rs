extern crate rand;

use std::time::Duration;

use rand::Rng;

// Enum representing the different types of parties
//#[derive(Copy, Clone)]
#[derive(PartialEq)]
pub enum TypeOfPlayer {
    Unset,
    Human,
    Robot,
}

// Struct representing the infos of the current Game
// Board is 361 cases long -> 19 x 19 Board
//#[derive(Copy, Clone)]
pub struct Player {
    pub player_type: TypeOfPlayer,
    pub nb_of_catch: isize,
    pub name: &'static str,
    pub time_spent: Duration,
}

impl Player {
    // Getter of the Game Instance
    pub fn new() -> Player {
        Player {
            player_type: TypeOfPlayer::Unset,
            nb_of_catch: 0,
            name: "",
            time_spent: Duration::new(0, 0),
        }
    }

    // Setter of the Game Instance
    pub fn set_infos(
        &mut self,
        player_type: TypeOfPlayer,
        name: &'static str,
    ) -> () {
        self.player_type = player_type;
        self.name = name
    }

    pub fn get_time(&self) -> String {
        format!(
            "Time spent : {}.{:03}",
            self.time_spent.as_secs(),
            self.time_spent.subsec_millis()
        )
    }

    pub fn set_time(&mut self, time: Duration) -> () {
        self.time_spent = time;
    }
}

// Aim of the function :
// Function that initializes the 2 players regarding the Gameplay chosen
pub fn initialize_players(nb_player: usize) -> (Player, Player) {
    let mut player1: Player = Player::new();
    let mut player2: Player = Player::new();
    match nb_player {
        0 => {
            player1.set_infos(TypeOfPlayer::Robot, "Robot1");
            player2.set_infos(TypeOfPlayer::Robot, "Robot2");
        }
        1 => {
            player1.set_infos(TypeOfPlayer::Human, "Human1");
            player2.set_infos(TypeOfPlayer::Robot, "Robot1");
        }
        2 => {
            player1.set_infos(TypeOfPlayer::Human, "Human1");
            player2.set_infos(TypeOfPlayer::Human, "Human2");
        }
        _ => {
            player1.set_infos(TypeOfPlayer::Unset, "Unset1");
            player2.set_infos(TypeOfPlayer::Unset, "Unset2");
        }
    };
    let mut rng = rand::thread_rng();

    match rng.gen_range(0, 2) {
        0 => (player1, player2),
        1 => (player2, player1),
        _ => unreachable!(),
    }
}
