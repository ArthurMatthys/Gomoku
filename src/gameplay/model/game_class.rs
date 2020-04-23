// Enum representing the different types of parties
#[derive(Copy, Clone)]
pub enum TypeOfParty {
    Unset,
    Standard,
    Pro,
    Swap,
    Swap2
}

// Struct representing the infos of the current Game
// Board is 361 cases long -> 19 x 19 Board
#[derive(Copy, Clone)]
pub struct Game {
    pub nb_of_player: usize,
    pub board: [Option<bool>; 361],
    type_of_party: TypeOfParty
}

impl Game {

    // Getter of the Game Instance
    pub fn new() -> Game {
        Game {
            nb_of_player: 0,
            board: [None; 361],
            type_of_party: TypeOfParty::Unset
        }
    }

    // Setter of the Game Instance
    pub fn set_infos(&mut self, nb_of_player: usize, party_type: TypeOfParty) -> () {
            self.nb_of_player = nb_of_player;
            self.type_of_party = party_type;
    }

}