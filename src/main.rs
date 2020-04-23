extern crate gtk;
use gui::controller::*;
use gui::model::*;

mod gui;

// Import all methods frorm board.rs
// use game_infos::*;
// mod game_infos;

// Aim of the function :
// Print debug for double_three analysis
//fn print_grid(board: [Option<bool>; 361]) -> () {
//    for x in 1..(board.len() + 1) {
//        match board[x - 1] {
//            Some(true) => print!("x"),
//            Some(false) => print!("o"),
//            None => print!("+"),
//        }
//        if x % 19 == 0 {
//            print!("\n")
//        }
//    }
//}

// Aim of the function :
// Function that manages the global gameplay
//fn play(mut game: gui::model::game_class::Game) -> &'static str {
//    let (mut player1, mut player2) = initialize_players(game);
//    game.board[19] = Some(false);
//    game.board[20] = Some(false);
//    game.board[22] = Some(false);
//
//    game.board[67] = Some(false);
//    game.board[68] = Some(false);
//    game.board[70] = Some(false);
//
//    game.board[97] = Some(false);
//    game.board[116] = Some(false);
//    game.board[135] = Some(false);
//
//    print_grid(game.board);
//    "end"
//    // loop {
//
//    // }
//}

fn main() {
    println!("=== Start Gomoku ===");
    start_gui::start_gui();
    //let game:Result<game_class::Game, &str> = collect_game_infos::collect_info_about_party();
}
