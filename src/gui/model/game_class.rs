use super::*;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
// Enum representing the different types of parties

pub const TYPEOFPARTY: [&str; 4] = ["Standard", "Pro", "Swap", "Swap2"];

#[derive(Copy, Clone)]
pub enum TypeOfParty {
    Unset,
    Standard,
    Pro,
    Swap,
    Swap2,
}

pub struct Gamerule {
    nb_of_player: AtomicUsize,
    type_of_party: AtomicUsize,
}

impl Gamerule {
    pub fn new(initial_nb: usize, initial_gamerule: usize) -> Gamerule {
        Gamerule {
            nb_of_player: AtomicUsize::new(initial_nb),
            type_of_party: AtomicUsize::new(initial_gamerule),
        }
    }

    pub fn get_player(&self) -> usize {
        self.nb_of_player.load(Ordering::SeqCst)
    }

    pub fn get_gamerule(&self) -> usize {
        self.type_of_party.load(Ordering::SeqCst)
    }

    pub fn get_gamerule_name(&self) -> &str {
        TYPEOFPARTY[self.type_of_party.load(Ordering::SeqCst)]
    }

    pub fn set_gamerule(&self, party_type: usize) -> () {
        self.type_of_party.store(party_type, Ordering::SeqCst);
    }

    pub fn set_nb_player(&self, nb_of_player: i32) {
        let current = self.get_player() as i32;
        if current + nb_of_player < 0 || current + nb_of_player > 2 {
        } else {
            self.nb_of_player
                .store((current + nb_of_player) as usize, Ordering::SeqCst);
        }
    }
}

// Struct representing the infos of the current Game
// Board is 361 cases long -> 19 x 19 Board
pub struct Game {
    pub nb_of_player: AtomicUsize,
    pub board: [Option<bool>; 361],
    type_of_party: AtomicUsize,
    pub players: AtomicPtr<&(player_class::Player, player_class::Player)>,
}

impl Game {
    // Getter of the Game Instance
    pub fn new() -> Game {
        Game {
            nb_of_player: AtomicUsize::new(0),
            board: [None; 361],
            type_of_party: AtomicUsize::new(TypeOfParty::Unset as usize),
            players: AtomicPtr::new(&player_class::initialize_players(3)),
        }
    }

    pub fn set_type_of_party(&self, type_of_party: TypeOfParty) {
        self.type_of_party
            .store(type_of_party as usize, Ordering::SeqCst);
    }

    pub fn get_type_of_party(&self) -> TypeOfParty {
        let party_type: TypeOfParty;
        match self.type_of_party.load(Ordering::SeqCst) {
            0 => party_type = TypeOfParty::Standard,
            1 => party_type = TypeOfParty::Pro,
            2 => party_type = TypeOfParty::Swap,
            3 => party_type = TypeOfParty::Swap2,
            _ => party_type = TypeOfParty::Standard,
        }
        party_type
    }

    pub fn set_nb_of_player(&self, nb_of_player: usize) {
        self.nb_of_player.store(nb_of_player, Ordering::SeqCst);
    }

    pub fn set_info(&self, nb_of_player: usize, type_of_party: usize) {
        let party_type: TypeOfParty;
        match type_of_party {
            0 => party_type = TypeOfParty::Standard,
            1 => party_type = TypeOfParty::Pro,
            2 => party_type = TypeOfParty::Swap,
            3 => party_type = TypeOfParty::Swap2,
            _ => party_type = TypeOfParty::Standard,
        }
        self.set_type_of_party(party_type);
        self.set_nb_of_player(nb_of_player);
        self.players
            .store(player_class::initialize_players(nb_of_player));
    }

    // Setter of the Game Instance
}
