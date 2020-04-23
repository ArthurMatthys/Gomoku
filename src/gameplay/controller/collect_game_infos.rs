use std::io::Write;
use std::io;
use std::str::FromStr;

use super::super::model::game_class;


fn retrieve_nb_human_players() -> Result<usize, &'static str> {
    let mut entree = String::new();
    match io::stdin().read_line(&mut entree) {
        Ok(_) => {
            match usize::from_str(&entree.trim()) {
                Ok(nombre) =>   if nombre == 1 ||  nombre == 2 || nombre == 0 { 
                                    Ok(nombre)
                                } else {
                                    Err("Wrong number entered")
                                },    
                Err(_) => {
                    Err("Error while parsing data")
                }
            }
        },
        _ => {
            Err("Error during the retrieval of the writing")
        }
    }
}

fn retrieve_party_type() -> Result<game_class::TypeOfParty, &'static str> {
    let mut entree:String = String::new();
    // Arrays
    let choices:[usize;4] =  [1, 2, 3, 4];
    let enum_types: [game_class::TypeOfParty; 4] = [game_class::TypeOfParty::Standard, game_class::TypeOfParty::Pro, game_class::TypeOfParty::Swap, game_class::TypeOfParty::Swap2];
    // Pattern match on cases. If working, retrieve. Else retrieve Err(str)
    match io::stdin().read_line(&mut entree) {
        Ok(_) => {
            match usize::from_str(&entree.trim()) {
                Ok(nombre) =>   if choices.contains(&nombre) { 
                                    Ok(enum_types[nombre - 1])
                                } else {
                                    Err("Wrong number entered")
                                },    
                Err(_) => {
                    Err("Error while parsing data")
                }
            }
        },
        _ => {
            Err("Error during the retrieval of the writing")
        }
    }
}

// Aim of the function :
// Collect all the infos necessary for the Board struct
pub fn collect_info_about_party() -> Result<game_class::Game, &'static str> {
    let mut game:game_class::Game = game_class::Game::new();
    print!("Number of human players |0, 1 or 2| : ");
    let _ = io::stdout().flush();
    let nb_of_players: Result<usize, &str> = retrieve_nb_human_players();
    print!("Number Type of Play |1 (Standard) / 2 (Pro) / 3 (Swap) / 4 (Swap2) | : ");
    let _ = io::stdout().flush();
    let party_type: Result<game_class::TypeOfParty, &str> = retrieve_party_type();
    match (nb_of_players, party_type) {
        (Ok(x), Ok(y)) => { game.set_infos(x,y); Ok(game) },
        (Err(x),_) => { Err(x) },
        (_,Err(y)) => { Err(y) }
    }
}