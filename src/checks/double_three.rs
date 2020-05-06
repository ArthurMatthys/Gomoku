use super::super::model::game;
use super::after_turn_check;
use super::capture;


fn is_free_tree(game: &mut game::Game, index: isize, current: bool, dir: isize) -> bool {
    let mut parts = [[0, 0, 0, 0], [0, 0, 0, 0]];
    for i in [-1, 1].iter() {
        let index_part: usize = ((i + 1) / 2) as usize;
        let mut moves: isize = 1;
        loop {
            if capture::valid_dir(&index, i * dir, moves) {
                let new_index = index + dir * moves * i;
                match game.board[new_index as usize] {
                    //  If I am on an empty position
                    None => {
                        // Check wether we already met an empty position
                        // If yes
                        if parts[index_part][1] == 1 {
                            if parts[index_part][1] == 1 && parts[index_part][2] != 0 {
                                parts[index_part][3] = 1;
                            }
                            break;
                        // Else, increment second index
                        } else {
                            parts[index_part][1] = 1;
                        }
                    }
                    // If I am on a competitors pawn, break
                    Some(x) if x != current => {
                        break;
                    }
                    // If I am on the player's pawn
                    Some(x) if x == current => {
                        // If I have met an empty position, increment the index 2 of the vec
                        if parts[index_part][1] == 1 {
                            parts[index_part][2] += 1;
                        // Else, increment the index on positon 0
                        } else {
                            parts[index_part][0] += 1;
                        }
                    }
                    _ => unreachable!(),
                };
                // Check next move
                moves += 1;
            // If we are on an invalid position, break
            } else {
                break;
            }
        }
    }
    let tot = (
        parts[0][0] + parts[1][0],
        parts[0][1] + parts[1][1],
        parts[0][2] + parts[1][2],
        parts[0][3] + parts[1][3],
    );
    if tot.3 >= 1
        && tot.1 == 2
        && (tot.0 + tot.2 == 2 && ((parts[0][2] * parts[1][2] == 0) || tot.0 != 0)) {
        true
    } else {
        false
    }
}

pub fn check_double_three(game: &mut game::Game) -> Vec<usize> {
    let mut ret = vec![];
    let pawn_current_player = !game
        .player_to_pawn()
        .expect("Could not retrieve player pawn");
    for i in 0..361 {
        let mut nbr_free_tree = 0;
        if game.board[i] != None {
            continue;
        } else {
            after_turn_check::DIRECTIONS.iter().for_each(|&x| {
                if is_free_tree(game, i as isize, pawn_current_player, x) {
                    nbr_free_tree += 1;
                }
            });
        }
        if nbr_free_tree >= 2 {
            ret.push(i);
        }
    }
    ret
}
