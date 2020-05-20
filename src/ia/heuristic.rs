use super::super::checks::after_turn_check::DIRECTIONS;

use super::super::render::board::SIZE_BOARD;

use super::super::model::player;

macro_rules! valid_coord {
    ($e:expr, $v:expr) => {
        $e >= 0 && $v >= 0 && ($e as usize) < SIZE_BOARD && ($v as usize) < SIZE_BOARD
    };
}

fn evaluate_board(
    board: [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
) -> [[[(i8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD] {
    let mut score_tab: [[[(i8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD] =
        [[[(-1, Some(false), Some(false)); 4]; SIZE_BOARD]; SIZE_BOARD];
    for x in 0..SIZE_BOARD {
        for y in 0..SIZE_BOARD {
            if let Some(player) = board[x][y] {
                // todo multithread
                for dir in 0..4 {
                    if score_tab[x][y][dir].0 != -1 {
                        continue;
                    } else {
                        let mut count = 1i8;
                        let mut block_left = Some(false);
                        let mut block_right = Some(false);
                        let mut indexes = vec![(x, y)];
                        let direction = DIRECTIONS[dir];
                        for way in [-1, 1].iter() {
                            let mut step = 1isize;
                            loop {
                                let new_x = x as isize + (way * step * direction.0);
                                let new_y = y as isize + (way * step * direction.1);
                                if valid_coord!(new_x, new_y) {
                                    if let Some(value) = board[new_x as usize][new_y as usize] {
                                        if value == player {
                                            count += 1;
                                            indexes.push((new_x as usize, new_y as usize));
                                        } else {
                                            if *way == -1 {
                                                block_left = Some(true);
                                            } else {
                                                block_right = Some(true);
                                            }
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                } else {
                                    if *way == -1 {
                                        block_left = None;
                                    } else {
                                        block_right = None;
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

const INSTANT_WIN: i64 = 100000000;
const TWO_STEP_WIN: i64 = 010000000;
const FOUR_STEP_WIN: i64 = 001000000;
const SIX_STEP_WIN: i64 = 000100000;
const EIGHT_STEP_WIN: i64 = 000010000;
const TEN_STEP_WIN: i64 = 000001000;
const SCORE_TAKE: i64 = 000000010;

fn score_to_points(
    nb_caught: isize,
    (
        nb_catch,
        nb_5,
        nb_5_take,
        nb_4_o,
        nb_4_so,
        nb_4_c,
        nb_3_o,
        nb_3_so,
        nb_3_c,
        nb_2_o,
        nb_2_so,
        nb_2_c,
    ): (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8),
    depth: u32,
) -> i64 {
    let mut total = 0i64;
    match nb_caught {
        4 => {
            if nb_catch > 1 {
                total += INSTANT_WIN;
            } else if nb_catch == 1 {
                total += TWO_STEP_WIN;
            }
        }
        a => total += SCORE_TAKE.pow((a as u8 + nb_catch / 2) as u32),
    }
    total += (nb_5 / 5) as i64 * INSTANT_WIN;
    total += (nb_5_take / 5) as i64 * TWO_STEP_WIN;

    total += (nb_4_o / 4) as i64 * TWO_STEP_WIN;
    total += (nb_4_so / 4) as i64 * FOUR_STEP_WIN;
    total -= (nb_4_c / 4) as i64 * SIX_STEP_WIN;

    total += (nb_3_o / 3) as i64 * FOUR_STEP_WIN;
    total += (nb_3_so / 3) as i64 * SIX_STEP_WIN;
    total -= (nb_3_c / 3) as i64 * EIGHT_STEP_WIN;

    total += (nb_2_o / 2) as i64 * SIX_STEP_WIN;
    total += (nb_2_so / 2) as i64 * EIGHT_STEP_WIN;
    total -= (nb_2_c / 2) as i64 * TEN_STEP_WIN;

    total * ((depth + 1) as i64 * 2)
}

pub fn first_heuristic(
    board: [[Option<bool>; SIZE_BOARD]; SIZE_BOARD],
    player_actual: player::Player,
    player_opposite: player::Player,
    depth: u32,
) -> i64 {
    // nb_of catch/5 in a row/5 in a row can take/4 open/4 semi-open/4 close
    // 3 open/3 semi-open/3 close/2 open/2 semi-open/2 close
    let good_points = (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
    let bad_points = (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
    let score_board: [[[(i8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD] =
        evaluate_board(board);

    let check_free_space = |dir: usize,
                            x: usize,
                            y: usize,
                            status_pawn: Option<bool>,
                            left: Option<bool>,
                            right: Option<bool>| {
        let actual_pawn = match status_pawn {
            Some(a) => a,
            None => unreachable!(),
        };
        let mut free_space = 0;
        for way in [-1, 1].iter() {
            let mut changed = 0;
            if way == &-1 && (left == None || left == Some(true)) {
                continue;
            }
            if way == &1 && (right == None || right == Some(true)) {
                continue;
            }
            let mut new_x = x;
            let mut new_y = y;
            loop {
                new_x += (way * DIRECTIONS[dir].0) as usize;
                new_y += (way * DIRECTIONS[dir].1) as usize;
                if new_x >= SIZE_BOARD || new_y >= SIZE_BOARD || board[new_x][new_y] != status_pawn
                {
                    continue;
                }
                match board[new_x][new_y] {
                    Some(a) if a != actual_pawn => break,
                    Some(a) if a == actual_pawn => {
                        if changed == 1 {
                            free_space += score_board[new_x][new_y][dir].0;
                        }
                    }
                    Some(_) => unreachable!(),
                    None => {
                        changed = 1;
                        free_space += 1
                    }
                };
            }
        }
        return free_space;
    };

    let handle_2 = |dir: usize,
                    x: usize,
                    y: usize,
                    status_pawn: Option<bool>,
                    left: Option<bool>,
                    right: Option<bool>| {
        if (left == None || left == Some(true)) && (right == None || right == Some(true)) {
            return 2;
        } else {
            let free_space = 2 + check_free_space(dir, x, y, status_pawn, left, right);
            if free_space >= 5 {
                if (left == None || left == Some(true)) || (right == None || right == Some(true)) {
                    return 1;
                } else {
                    return 0;
                }
            } else {
                return 2;
            }
        }
    };

    let handle_3 = |dir: usize,
                    x: usize,
                    y: usize,
                    status_pawn: Option<bool>,
                    left: Option<bool>,
                    right: Option<bool>| {
        if (left == None || left == Some(true)) && (right == None || right == Some(true)) {
            return 2;
        } else {
            let free_space = 3 + check_free_space(dir, x, y, status_pawn, left, right);
            if free_space >= 5 {
                if (left == None || left == Some(true)) || (right == None || right == Some(true)) {
                    return 1;
                } else {
                    return 0;
                }
            } else {
                return 2;
            }
        }
    };

    let handle_4 = |left, right| {
        let mut block_left = false;
        let mut block_right = false;
        if let Some(a) = left {
            if a {
                block_left = true;
            }
        } else {
            block_left = true;
        }
        if let Some(b) = right {
            if b {
                block_right = true;
            }
        } else {
            block_right = true;
        }
        if block_right && block_left {
            return 2;
        } else if block_right || block_left {
            return 1;
        } else {
            return 0;
        }
    };

    let handle_5 = |dir: usize, x: usize, y: usize, status_pawn| {
        for new_dir in 0..4 {
            if dir == new_dir {
                continue;
            } else {
                match score_board[x][y][dir] {
                    (2, Some(true), Some(false)) => return true,
                    (2, Some(false), Some(true)) => return true,
                    _ => continue,
                }
            }
        }
        for way in [-1, 1].iter() {
            let mut new_x = x;
            let mut new_y = y;
            loop {
                new_x += (way * DIRECTIONS[dir].0) as usize;
                new_y += (way * DIRECTIONS[dir].1) as usize;
                if new_x >= SIZE_BOARD || new_y >= SIZE_BOARD || board[new_x][new_y] != status_pawn
                {
                    continue;
                } else {
                    for new_dir in 0..4 {
                        if new_dir == dir {
                            continue;
                        } else {
                            match score_board[new_x][new_y][new_dir] {
                                (2, Some(true), Some(false)) => return true,
                                (2, Some(false), Some(true)) => return true,
                                _ => continue,
                            }
                        }
                    }
                }
            }
        }
        return false;
    };

    let count_points = |mut count: (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8),
                        mut opp_count: (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8),
                        x: usize,
                        y: usize,
                        status_pawn|
     -> () {
        for dir in 0..4 {
            match score_board[x][y][dir] {
                (1, _, _) => continue,
                (2, left, right) => match handle_2(dir, x, y, status_pawn, left, right) {
                    0 => count.9 += 1,
                    1 => {
                        count.10 += 1;
                        opp_count.0 += 1
                    }
                    2 => count.11 += 1,
                    _ => unreachable!(),
                },
                (3, left, right) => match handle_3(dir, x, y, status_pawn, left, right) {
                    0 => count.6 += 1,
                    1 => count.7 += 1,
                    2 => count.8 += 1,
                    _ => unreachable!(),
                },
                (4, left, right) => {
                    match handle_4(left, right) {
                        0 => count.3 += 1,
                        1 => count.4 += 1,
                        2 => count.5 += 1,
                        _ => unreachable!(),
                    };
                }
                (5..=10, _, _) => {
                    if handle_5(dir, x, y, status_pawn) {
                        count.2 += 1;
                    } else {
                        count.1 += 1;
                    }
                }
                _ => unreachable!(),
            }
        }
    };
    for x in 0..SIZE_BOARD {
        for y in 0..SIZE_BOARD {
            match board[x][y] {
                None => continue,
                e => {
                    if e == player_actual.bool_type {
                        count_points(bad_points, good_points, x, y, e);
                    } else {
                        count_points(good_points, bad_points, x, y, e);
                    }
                }
            };
        }
    }

    score_to_points(player_opposite.nb_of_catch, good_points, depth)
        - score_to_points(player_actual.nb_of_catch, bad_points, depth)
}
