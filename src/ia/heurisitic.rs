use super::super::render::board::SIZE_BOARD;

pub const DIRECTIONS: [(isize, isize); 4] = [(1, 1), (1, 0), (0, 1), (1, -1)];

macro_rules! valid_coord {
    ($e:expr, $v:expr) => {
        $e >= 0 && $v >= 0 && ($e as usize) < SIZE_BOARD && ($v as usize) < SIZE_BOARD
    };
}

fn evaluate_board(
    board: [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
) -> [[[(i32, bool, bool); 4]; SIZE_BOARD]; SIZE_BOARD] {
    let mut score_tab: [[[(i32, bool, bool); 4]; SIZE_BOARD]; SIZE_BOARD] =
        [[[(-1, false, false); 4]; SIZE_BOARD]; SIZE_BOARD];
    for x in 0..SIZE_BOARD {
        for y in 0..SIZE_BOARD {
            if let Some(player) = board[x][y] {
                // todo multithread
                for dir in 0..4 {
                    if score_tab[x][y][dir].0 != -1 {
                        continue;
                    } else {
                        let mut count = 1i32;
                        let mut block_left = false;
                        let mut block_right = false;
                        let mut indexes = vec![(x, y)];
                        let direction = DIRECTIONS[dir];
                        for way in [-1, 1].iter() {
                            let mut step = 1isize;
                            loop {
                                let new_x = x as isize + (way * step * direction.0);
                                let new_y = y as isize + (way * step * direction.1);
                                if valid_coord!(new_x, new_y) {
                                    if let Some(value) = board[x][y] {
                                        if value == player {
                                            count += 1;
                                            indexes.push((new_x as usize, new_y as usize));
                                        } else {
                                            if *way == -1 {
                                                block_left = true;
                                            } else {
                                                block_right = true;
                                            }
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                } else {
                                    if *way == -1 {
                                        block_left = true;
                                    } else {
                                        block_right = true;
                                    }
                                    break;
                                }
                                step += 1;
                            }
                        }
                        indexes.iter().for_each(|&(x, y)| {
                            score_tab[x][y][dir] = (count, block_left, block_right)
                        });
                    }
                }
            }
        }
    }
    score_tab
}

pub fn first_heuristic(
    board: [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    player: Option<bool>,
) -> i64 {
    let value_heuristic = 0i64;

    value_heuristic
}
