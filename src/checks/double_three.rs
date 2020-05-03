use super::super::model::game;
use super::after_turn_check;
use super::capture;

fn      count_again_until_none_or_curr_position(
            board: &[Option<bool>; 361],
            index: &isize,
            direction: isize,
            counter: usize,
            pawn_current_player: &bool)
        -> usize {
    if *index >= 0 && *index < 361 && board[*index as usize] == None {
        counter
    } else if *index >= 0 && *index < 361 && board[*index as usize] == Some(*pawn_current_player) {
        count_again_until_none_or_curr_position(
            board,
            &(*index + direction),
            direction,
            counter + 1,
            pawn_current_player)
    } else { 0 }
}

fn      free_three_present_at_index(
            board: &[Option<bool>; 361],
            direction: isize,
            index_orig: &usize,
            index: &isize,
            pawn_current_player: &bool,
            counter: usize,
            life: &usize)
        -> usize {
    // If the position is at None
    if *index >= 0 && *index < 361 && board[*index as usize] == None {
        // If I have a life and can use it, advance and consume life 
        if *life == 1 && counter <= 2 {
            // println!("{}", "je recurse");
            free_three_present_at_index(
                board,
                direction,
                index_orig,
                &(*index + direction),
                pawn_current_player,
                counter,
                &(*life - 1)
            )
        // If I am on a None position, return counter
        } else { counter }
    // If the position has the same color as the player's color
    } else if *index >= 0 && *index < 361 && board[*index as usize] == Some(*pawn_current_player) {
            free_three_present_at_index(
                board,
                direction,
                index_orig,
                &(*index + direction),
                pawn_current_player,
                counter + 1,
                life
            )
    // If the position is contradictory with the player's color
    } else if *index >= 0 && *index < 361 && board[*index as usize] == Some(!*pawn_current_player) {
        count_again_until_none_or_curr_position(board, &(*index_orig as isize + direction), direction, 0, pawn_current_player)
    } else {
    //     println!("{}", "oups");
    //     unreachable!();
    // }
    // If index is out of board, do as if we met a black-board
        count_again_until_none_or_curr_position(board, &(*index_orig as isize + direction), direction, 0, pawn_current_player) }
}

// Aim of the function :
// Easily avoid positions where a pawn is already placed
// Easier to manage here than in recursive
fn      index_available(
            board: &[Option<bool>; 361],
            direction: isize,
            index_orig: &usize,
            index: &isize,
            pawn_current_player: &bool,
            counter: usize)
        -> usize {
if *index >= 0 && *index < 361 && board[*index as usize] == None { 
    // let (count, count2, life) = (0,0,0);
    // let life = 1;
    // println!("Life_before:{}", life);
    let count = free_three_present_at_index(board, direction, index_orig, &(index + direction), pawn_current_player, counter, &1);
    // println!("Life_middle:{}", life);
    let count2 = free_three_present_at_index(board, -direction, index_orig, &(index - direction), pawn_current_player, counter, &1);
    // println!("Life_after:{}", life);
    if capture::valid_dir(index, direction, count as isize) { count } else { 0 };
    if capture::valid_dir(index, -direction, count2 as isize) { count2 } else { 0 };
    // Faire dans les 2 directions
    // Ne pas oublier la nouvelle fonction d'Arthur / sa nouvelle problématique -> dépassement du boardj,,
    // Ressort le nombre de pions valides pour free_three dans la direction
    count + count2
    // Addition des 2, et si c'est == 
    } else { 0 }
}

fn      double_three_present(index: &isize, pawn_current_player: &bool, board: &[Option<bool>; 361]) -> bool {
    // Compter le nb de free_three
    // Un free_three, c'est quand la somme de ce que l'on rend == 2
    // println!{"DEBUG: {}, ", index_available(board, )}
    let free_threes_multi_direction = after_turn_check::DIRECTIONS.iter().filter(|&x| index_available(board, *x, &(*index as usize), index, pawn_current_player, 0) == 2).collect::<Vec<&isize>>();
    // let free_threes_same_direction = after_turn_check::DIRECTIONS.iter().filter(|&x| index_available(board, *x, &(*index as usize), index, pawn_current_player, 0) == 4).collect::<Vec<&isize>>();
    // free_threes_multi_direction.len() > 1 || free_threes_same_direction.len() > 0
    free_threes_multi_direction.len() > 1
}

pub fn  forbidden_indexes(game:&game::Game) -> Option<Vec<usize>> {
    if let Some(pawn_current_player) = game.player_to_pawn() {
        let all_results = game.board.iter().enumerate().filter(|(i, _)| double_three_present(&(*i as isize), &!pawn_current_player, &game.board)).collect::<Vec<(usize, &Option<bool>)>>();
        Some(all_results.iter().map(|(i, _)| *i).collect::<Vec<usize>>())
    } else { None }
}