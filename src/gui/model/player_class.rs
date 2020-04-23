use super::*;
// Enum representing the different types of parties
#[derive(Copy, Clone)]
pub enum TypeOfPlayer {
    Unset,
    Human,
    Robot,
}

// Struct representing the infos of the current Game
// Board is 361 cases long -> 19 x 19 Board
#[derive(Copy, Clone)]
pub struct Player {
    player_type: TypeOfPlayer,
    pub nb_of_catch: isize,
    pub bool_type: Option<bool>,
    name: &'static str,
}

impl Player {
    // Getter of the Game Instance
    pub fn new() -> Player {
        Player {
            player_type: TypeOfPlayer::Unset,
            nb_of_catch: 0,
            bool_type: None,
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
}

// Aim of the function :
// Function that initializes the 2 players regarding the Gameplay chosen
pub fn initialize_players(nb_player: usize) -> (player_class::Player, player_class::Player) {
    let mut player1: player_class::Player = player_class::Player::new();
    let mut player2: player_class::Player = player_class::Player::new();
    match nb_player {
        0 => {
            player1.set_infos(player_class::TypeOfPlayer::Robot, Some(true), "Robot1");
            player2.set_infos(player_class::TypeOfPlayer::Robot, Some(false), "Robot2");
        }
        1 => {
            player1.set_infos(player_class::TypeOfPlayer::Human, Some(true), "Human1");
            player2.set_infos(player_class::TypeOfPlayer::Robot, Some(false), "Robot1");
        }
        2 => {
            player1.set_infos(player_class::TypeOfPlayer::Human, Some(true), "Human1");
            player2.set_infos(player_class::TypeOfPlayer::Human, Some(false), "Human2");
        }
        _ => {
            player1.set_infos(player_class::TypeOfPlayer::Unset, Some(true), "Unset1");
            player2.set_infos(player_class::TypeOfPlayer::Unset, Some(false), "Unset2");
        }
    };
    (player1, player2)
}
