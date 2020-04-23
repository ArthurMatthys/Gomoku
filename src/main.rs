// Import all methods frorm board.rs
// use game_infos::*;
// mod game_infos;
use gameplay::model::*;
use gameplay::controller::*;
mod gameplay;

// Aim of the function :
// Function that initializes the 2 players regarding the Gameplay chosen
fn initialize_players(game:gameplay::model::game_class::Game) -> (player_class::Player, player_class::Player)  {
    let mut player1:player_class::Player = player_class::Player::new();
    let mut player2:player_class::Player = player_class::Player::new();
    match game {
        game_class::Game { nb_of_player:0, .. } => {
            player1.set_infos(player_class::TypeOfPlayer::Robot, Some(true), "Robot1");
            player2.set_infos(player_class::TypeOfPlayer::Robot, Some(false), "Robot2");
            (player1,player2)
        },
        game_class::Game { nb_of_player:1, .. } => { 
            player1.set_infos(player_class::TypeOfPlayer::Human, Some(true), "Human1");
            player2.set_infos(player_class::TypeOfPlayer::Robot, Some(false), "Robot1");
            (player1,player2)
         },
        game_class::Game { nb_of_player:2, .. } => { 
            player1.set_infos(player_class::TypeOfPlayer::Human, Some(true), "Human1");
            player2.set_infos(player_class::TypeOfPlayer::Human, Some(false), "Human2");
            (player1,player2)
        },
        game_class::Game { .. } => {
            (player1,player2)
        }
    }
}

// Aim of the function :
// Print debug for double_three analysis
fn  print_grid(board: [Option<bool>; 361]) -> () {
    for x in 1..(board.len() + 1) {
        match board[x - 1] {
            Some(true) => { print!("x") },
            Some(false) => { print!("o") },
            None => { print!("+") }
        }
        if x % 19 == 0 {print!("\n")}
    }
}

// Aim of the function :
// Function that manages the global gameplay
fn play(mut game:gameplay::model::game_class::Game) -> &'static str {
    let (mut player1, mut player2) = initialize_players(game);
    game.board[19] = Some(false);
    game.board[20] = Some(false);
    game.board[22] = Some(false);

    game.board[67] = Some(false);
    game.board[68] = Some(false);
    game.board[70] = Some(false);

    game.board[97] = Some(false);
    game.board[116] = Some(false);
    game.board[135] = Some(false);

    print_grid(game.board);
    "end"
    // loop {

    // }
}

fn main() {
    println!("=== Start Gomoku ===");
    let game:Result<game_class::Game, &str> = collect_game_infos::collect_info_about_party();
    match game {
        Ok(game) => { println!("{}", play(game)) },
        Err(x) => {  println!("{}",x) }
    }
}
