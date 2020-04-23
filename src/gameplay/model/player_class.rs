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
    pub bool_type: Option <bool>,
    name: &'static str
}

impl Player {

    // Getter of the Game Instance
    pub fn new() -> Player {
        Player {
            player_type: TypeOfPlayer::Unset,
            nb_of_catch: 0,
            bool_type: None,
            name: ""
        }
    }

    // Setter of the Game Instance
    pub fn set_infos(&mut self, player_type: TypeOfPlayer, bool_type: Option<bool>, name: &'static str) -> () {
            self.player_type = player_type;
            self.bool_type = bool_type;
            self.name = name
    }

}