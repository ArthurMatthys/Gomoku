use super::super::checks::after_turn_check::DIRECTIONS;
use super::super::model::board::Board;
use super::super::model::score_board::ScoreBoard;
use std::thread::sleep;
use std::time::Duration;

use super::super::render::board::SIZE_BOARD;

pub fn evaluate_board(board: &mut Board) -> ScoreBoard {
    let mut score_tab: [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD] =
        [[[(0, Some(false), Some(false)); 4]; SIZE_BOARD]; SIZE_BOARD];
    for x in 0..SIZE_BOARD {
        for y in 0..SIZE_BOARD {
            if let Some(player) = board.get_pawn(x, y) {
                // todo multithread
                for dir in 0..4 {
                    if score_tab[x][y][dir].0 != 0 {
                        continue;
                    } else {
                        let mut count = 1u8;
                        let mut block_left = Some(false);
                        let mut block_right = Some(false);
                        let mut indexes = Vec::with_capacity(15);
                        indexes.push((x, y));
                        let direction = DIRECTIONS[dir];
                        for way in [-1, 1].iter() {
                            let mut step = 1isize;
                            loop {
                                let new_x = x as isize + (way * step * direction.0);
                                let new_y = y as isize + (way * step * direction.1);
                                match board.get(new_x as usize, new_y as usize) {
                                    Some(None) => break,
                                    Some(Some(a)) if a == player => {
                                        count += 1;
                                        indexes.push((new_x as usize, new_y as usize));
                                    }
                                    Some(Some(a)) if a != player => {
                                        if *way == -1 {
                                            block_left = Some(true);
                                        } else {
                                            block_right = Some(true);
                                        }
                                        break;
                                    }
                                    Some(Some(_)) => unreachable!(),
                                    None => {
                                        if *way == -1 {
                                            block_left = None;
                                        } else {
                                            block_right = None;
                                        }
                                        break;
                                    }
                                };
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
    score_tab.into()
}

pub const INSTANT_WIN: i64 = 1000000000000000;
const ALIGN_2: i64 = 10;
const ALIGN_3: i64 = 100;
const ALIGN_4: i64 = 1000;
const SCORE_TAKE: i64 = 10;
const FIVE_CAN_TAKE: i64 = 0010000;
const SCORE_CAN_TAKE: i64 = 000000100;
const BONUS_OLD_MOOVE: i64 = 1000;
// pub const MULTIPLIER: i64 = 10;

fn score_to_points(
    nb_caught: &mut isize,
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
    depth: &mut i8,
    last_player: bool,
) -> i64 {
    let mut total = 0i64;
    match *nb_caught {
        5..=8 => return INSTANT_WIN * ((*depth + 1) as i64 * 2),
        4 => {
            if nb_catch > 2 {
                return INSTANT_WIN * ((*depth + 1) as i64 * 2);
            } else if nb_catch == 2 {
                total += ALIGN_4;
            }
        }
        a => total += SCORE_TAKE * (a as i64 * SCORE_CAN_TAKE) as i64,
    }
    if nb_5 > 0 {
        return INSTANT_WIN * ((*depth + 1) as i64 * 2);
    }
    if last_player {
        total += (nb_5_take / 5) as i64 * FIVE_CAN_TAKE;

        total += (nb_4_o / 4) as i64 * ALIGN_4;
        total += (nb_4_so / 4) as i64 * ALIGN_4 / 2;

        total += (nb_3_o / 3) as i64 * ALIGN_3;
    } else {
        total += BONUS_OLD_MOOVE * (nb_5_take / 5) as i64 * FIVE_CAN_TAKE;

        total += BONUS_OLD_MOOVE * (nb_4_o / 4) as i64 * ALIGN_4;
        total += BONUS_OLD_MOOVE * (nb_4_so / 4) as i64 * ALIGN_4 / 2;

        total += BONUS_OLD_MOOVE * (nb_3_o / 3) as i64 * ALIGN_3;
    }

    total -= (nb_4_c / 4) as i64 * ALIGN_4 / 4;

    total += (nb_3_so / 3) as i64 * ALIGN_3 / 2;
    total -= (nb_3_c / 3) as i64 * ALIGN_3 / 4;

    total += (nb_2_o / 2) as i64 * ALIGN_2;
    total -= (nb_2_so / 2) as i64 * ALIGN_3 / 2;
    total -= (nb_2_c / 2) as i64 * ALIGN_2 / 4;

    total * ((*depth * 100 + 1) as i64)
}

pub fn first_heuristic_hint(
    board: &mut Board,
    score_board: &mut ScoreBoard,
    player_actual: Option<bool>,
    player_actual_catch: &mut isize,
    player_opposite_catch: &mut isize,
    depth: &mut i8,
) -> i64 {
    let (good_points, bad_points) = get_alignements(board, score_board, player_actual);

    score_to_points(player_actual_catch, good_points, depth, false)
        - score_to_points(player_opposite_catch, bad_points, depth, true)
}

fn get_alignements(
    board: &mut Board,
    score_board: &mut ScoreBoard,
    player_actual: Option<bool>,
) -> (
    (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8),
    (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8),
) {
    // nb_of catch/5 in a row/5 in a row can take/4 open/4 semi-open/4 close
    // 3 open/3 semi-open/3 close/2 open/2 semi-open/2 close
    let mut good_points = (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
    let mut bad_points = (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);

    let check_free_space = |dir: usize,
                            x: usize,
                            y: usize,
                            start_align: u8,
                            status_pawn: Option<bool>,
                            left: Option<bool>,
                            right: Option<bool>| {
        let actual_pawn = match status_pawn {
            Some(a) => a,
            None => unreachable!(),
        };
        let mut free_space = start_align;
        for way in [-1, 1].iter() {
            let mut changed = 0;
            if way == &-1 && (left == None || left == Some(true)) {
                continue;
            }
            if way == &1 && (right == None || right == Some(true)) {
                continue;
            }
            let mut new_x = x as isize;
            let mut new_y = y as isize;
            loop {
                if free_space >= 5 {
                    break;
                }
                new_x += way * DIRECTIONS[dir].0;
                new_y += way * DIRECTIONS[dir].1;
                match board.get(new_x as usize, new_y as usize) {
                    Some(None) => {
                        changed = 1;
                        free_space += 1
                    }
                    Some(Some(a)) if a != actual_pawn => break,
                    Some(Some(a)) if a == actual_pawn => {
                        if changed == 1 {
                            free_space +=
                                score_board.get(new_x as usize, new_y as usize, dir).0 as u8;
                        }
                    }
                    Some(Some(_)) => unreachable!(),
                    None => break,
                }
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
            let free_space = check_free_space(dir, x, y, 2, status_pawn, left, right);
            if free_space >= 5 {
                if (left == Some(false) && right == Some(true))
                    || (left == Some(true) && right == Some(false))
                {
                    return 1;
                }
                if left == None || left == Some(true) || right == None || right == Some(true) {
                    return 3;
                } else {
                    return 0;
                }
            } else {
                if (left == Some(false) && right == Some(true))
                    || (left == Some(true) && right == Some(false))
                {
                    return 4;
                } else {
                    return 2;
                }
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
            let free_space = check_free_space(dir, x, y, 3, status_pawn, left, right);
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

    let handle_5 = |dir: usize, x: usize, y: usize, status_pawn: Option<bool>| {
        for new_dir in 0..4 {
            if dir == new_dir {
                continue;
            } else {
                match score_board.get(x, y, new_dir) {
                    (2, Some(true), Some(false)) => return true,
                    (2, Some(false), Some(true)) => return true,
                    _ => continue,
                }
            }
        }
        for way in [-1, 1].iter() {
            let mut new_x = x as isize;
            let mut new_y = y as isize;

            loop {
                new_x += way * DIRECTIONS[dir].0;
                new_y += way * DIRECTIONS[dir].1;
                match board.get(new_x as usize, new_y as usize) {
                    None => {
                        // if new_x < 0|| new_y < 0 {
                        //         break;
                        //     }
                        break;
                        // ()
                    }
                    Some(a) if a == status_pawn => {
                        for new_dir in 0..4 {
                            if new_dir == dir {
                                continue;
                            } else {
                                match score_board.get(new_x as usize, new_y as usize, new_dir) {
                                    (2, Some(true), Some(false)) => return true,
                                    (2, Some(false), Some(true)) => return true,
                                    _ => continue,
                                }
                            }
                        }
                    }
                    Some(a) if a != status_pawn => {
                        break;
                    }
                    Some(_) => unreachable!(),
                }
            }
        }
        return false;
    };

    let count_points = |actual_tuple: &mut (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8),
                        opp_tuple: &mut (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8),

                        x: usize,
                        y: usize,
                        status_pawn: Option<bool>|
     -> () {
        for dir in 0..4 {
            match score_board.get(x, y, dir) {
                (1, _, _) => continue,
                (2, left, right) => match handle_2(dir, x, y, status_pawn, left, right) {
                    0 => actual_tuple.9 += 1,
                    1 => {
                        actual_tuple.10 += 1;
                        opp_tuple.0 += 1
                    }
                    2 => actual_tuple.11 += 1,
                    3 => actual_tuple.10 += 1,
                    4 => {
                        actual_tuple.11 += 1;
                        opp_tuple.0 += 1
                    }
                    _ => unreachable!(),
                },
                (3, left, right) => match handle_3(dir, x, y, status_pawn, left, right) {
                    0 => actual_tuple.6 += 1,
                    1 => actual_tuple.7 += 1,
                    2 => actual_tuple.8 += 1,
                    _ => unreachable!(),
                },
                (4, left, right) => {
                    match handle_4(left, right) {
                        0 => actual_tuple.3 += 1,
                        1 => actual_tuple.4 += 1,
                        2 => actual_tuple.5 += 1,
                        _ => unreachable!(),
                    };
                }
                (5..=9, _, _) => {
                    if handle_5(dir, x, y, status_pawn) {
                        actual_tuple.2 += 1;
                    } else {
                        actual_tuple.1 += 1;
                    }
                }
                (a, _, _) => {
                    board.print();
                    score_board.print();
                    println!("len : {}, ({},{})", a, x, y);
                    sleep(Duration::new(15, 1_000_000_000u32 / 60));
                    unreachable!()
                }
            }
        }
    };
    for x in 0..SIZE_BOARD {
        for y in 0..SIZE_BOARD {
            match board.get_pawn(x, y) {
                None => continue,
                a if a == player_actual => {
                    count_points(&mut good_points, &mut bad_points, x, y, a);
                }
                a if a != player_actual => {
                    count_points(&mut bad_points, &mut good_points, x, y, a);
                }
                Some(_) => unreachable!(),
            };
        }
    }
    (good_points, bad_points)
}

#[cfg(test)]
mod tests {
    use super::super::handle_board::change_score_board_add;
    use super::*;

    fn test_equals(
        t1: (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8),
        t2: (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8),
    ) -> bool {
        t1.0 == t2.0
            && t1.1 == t2.1
            && t1.2 == t2.2
            && t1.3 == t2.3
            && t1.4 == t2.4
            && t1.5 == t2.5
            && t1.6 == t2.6
            && t1.7 == t2.7
            && t1.8 == t2.8
            && t1.9 == t2.9
            && t1.10 == t2.10
            && t1.11 == t2.11
    }

    fn test_board(
        white_pos: Vec<(usize, usize)>,
        black_pos: Vec<(usize, usize)>,
        t1: (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8),
        t2: (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8),
    ) -> bool {
        let mut test_board: Board = [[None; SIZE_BOARD]; SIZE_BOARD].into();
        let mut score_tab: ScoreBoard =
            [[[(0, Some(false), Some(false)); 4]; SIZE_BOARD]; SIZE_BOARD].into();
        white_pos.iter().for_each(|&(x, y)| {
            test_board.set(x, y, Some(true));
            change_score_board_add(&mut test_board, &mut score_tab, x, y, Some(true));
        });
        black_pos.iter().for_each(|&(x, y)| {
            test_board.set(x, y, Some(false));
            change_score_board_add(&mut test_board, &mut score_tab, x, y, Some(false));
        });
        test_board.print();
        let (v1, v2) = get_alignements(&mut test_board, &mut score_tab, Some(false));
        let print_tuple = |(a, b, c, d, e, f, g, h, i, j, k, l): (
            u8,
            u8,
            u8,
            u8,
            u8,
            u8,
            u8,
            u8,
            u8,
            u8,
            u8,
            u8,
        )| {
            println!(
                "{:3},{:3},{:3},{:3},{:3},{:3},{:3},{:3},{:3},{:3},{:3},{:3},",
                a, b, c, d, e, f, g, h, i, j, k, l,
            )
        };
        print_tuple(v1);
        print_tuple(v2);
        let (vec1, vec2) = get_alignements(&mut test_board, &mut score_tab, Some(true));
        test_equals(v1, t1) && test_equals(v2, t2) && test_equals(vec1, t2) && test_equals(vec2, t1)
    }

    // nb_of catch/5 in a row/5 in a row can take/4 open/4 semi-open/4 close
    // 3 open/3 semi-open/3 close/2 open/2 semi-open/2 close
    #[test]
    fn test_twos_0() {
        let black_pos = vec![(8, 8), (8, 9), (9, 8)];
        let white_pos = vec![];
        assert!(test_board(
            white_pos,
            black_pos,
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0),
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        ));
    }
    #[test]
    fn test_twos_1() {
        let black_pos = vec![(8, 8), (8, 9), (9, 8)];
        let white_pos = vec![(8, 7)];
        assert!(test_board(
            white_pos,
            black_pos,
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 2, 0),
            (2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        ));
    }
    #[test]
    fn test_twos_2() {
        let black_pos = vec![(8, 8), (8, 9), (9, 8)];
        let white_pos = vec![(8, 7), (8, 10)];
        assert!(test_board(
            white_pos,
            black_pos,
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 2),
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        ));
    }
    #[test]
    fn test_twos_3() {
        let black_pos = vec![(8, 8), (8, 9), (9, 8)];
        let white_pos = vec![(8, 7), (8, 10), (10, 8)];
        assert!(test_board(
            white_pos,
            black_pos,
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2),
            (2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        ));
    }
    #[test]
    fn test_twos_4() {
        let black_pos = vec![(8, 8), (8, 9), (9, 8), (9, 9)];
        let white_pos = vec![];
        assert!(test_board(
            white_pos,
            black_pos,
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0),
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        ));
    }
    #[test]
    fn test_twos_5() {
        let black_pos = vec![(8, 8)];
        let white_pos = vec![
            (8, 9),
            (8, 10),
            (9, 8),
            (10, 8),
            (8, 7),
            (8, 6),
            (7, 8),
            (6, 8),
        ];
        assert!(test_board(
            white_pos,
            black_pos,
            (8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 8, 0),
        ));
    }
    #[test]
    fn test_twos_6() {
        let black_pos = vec![(8, 8), (8, 9)];
        let white_pos = vec![(7, 8), (6, 8)];
        assert!(test_board(
            white_pos,
            black_pos,
            (2, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0),
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0),
        ));
    }
    #[test]
    fn test_twos_7() {
        let black_pos = vec![(8, 8), (9, 8)];
        let white_pos = vec![(6, 8), (11, 8)];
        assert!(test_board(
            white_pos,
            black_pos,
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2),
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        ));
    }
    #[test]
    fn test_twos_8() {
        let black_pos = vec![(1, 1), (1, 2)];
        let white_pos = vec![(1, 4)];
        assert!(test_board(
            white_pos,
            black_pos,
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2),
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        ));
    }

    #[test]
    fn test_threes_0() {
        let black_pos = vec![(8, 8), (8, 9), (8, 10)];
        let white_pos = vec![];
        assert!(test_board(
            white_pos,
            black_pos,
            (0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0),
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        ));
    }
    #[test]
    fn test_threes_1() {
        let black_pos = vec![(8, 8), (8, 9), (8, 10)];
        let white_pos = vec![(8, 7)];
        assert!(test_board(
            white_pos,
            black_pos,
            (0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0),
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        ));
    }
    #[test]
    fn test_threes_2() {
        let black_pos = vec![(8, 8), (8, 9), (8, 10)];
        let white_pos = vec![(8, 7), (8, 11)];
        assert!(test_board(
            white_pos,
            black_pos,
            (0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0),
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        ));
    }
    #[test]
    fn test_threes_3() {
        let black_pos = vec![(8, 8), (8, 9), (8, 10), (9, 8), (9, 9), (9, 10)];
        let white_pos = vec![];
        assert!(test_board(
            white_pos,
            black_pos,
            (0, 0, 0, 0, 0, 0, 6, 0, 0, 14, 0, 0),
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        ));
    }
    #[test]
    fn test_threes_4() {
        let black_pos = vec![(8, 8), (8, 9), (8, 10), (9, 8), (9, 9), (9, 10)];
        let white_pos = vec![(8, 7), (7, 7)];
        assert!(test_board(
            white_pos,
            black_pos,
            (0, 0, 0, 0, 0, 0, 3, 3, 0, 12, 2, 0),
            (2, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0),
        ));
    }
    #[test]
    fn test_threes_5() {
        let black_pos = vec![(8, 8), (8, 9), (8, 10), (9, 8), (9, 9), (9, 10)];
        let white_pos = vec![(8, 7), (7, 7), (8, 12)];
        assert!(test_board(
            white_pos,
            black_pos,
            (0, 0, 0, 0, 0, 0, 3, 0, 3, 12, 2, 0),
            (2, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0),
        ));
    }
    #[test]
    fn test_threes_6() {
        let black_pos = vec![(1, 1), (1, 2), (1, 3)];
        let white_pos = vec![(1, 4)];
        assert!(test_board(
            white_pos,
            black_pos,
            (0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0),
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        ));
    }
    #[test]
    fn test_threes_7() {
        let black_pos = vec![(2, 2), (1, 3), (0, 4)];
        let white_pos = vec![];
        assert!(test_board(
            white_pos,
            black_pos,
            (0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0),
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        ));
    }
    #[test]
    fn test_threes_8() {
        let black_pos = vec![(0, 3), (1, 2), (2, 1)];
        let white_pos = vec![];
        assert!(test_board(
            white_pos,
            black_pos,
            (0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0),
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        ));
    }

    //
    #[test]
    fn test_fives_0() {
        let black_pos = vec![(8, 6), (8, 7), (8, 8), (8, 9), (8, 10)];
        let white_pos = vec![];
        assert!(test_board(
            white_pos,
            black_pos,
            (0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        ));
    }
    #[test]
    fn test_fives_1() {
        let black_pos = vec![(8, 6), (8, 7), (8, 8), (9, 8), (8, 9), (8, 10)];
        let white_pos = vec![(10, 8)];
        assert!(test_board(
            white_pos,
            black_pos,
            (0, 0, 5, 0, 0, 0, 0, 0, 0, 4, 2, 0),
            (2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        ));
    }
    #[test]
    fn test_fives_2() {
        let black_pos = vec![
            (8, 6),
            (8, 7),
            (8, 8),
            (9, 8),
            (8, 9),
            (8, 10),
            (6, 6),
            (6, 7),
            (6, 8),
            (6, 9),
            (6, 10),
        ];
        let white_pos = vec![(10, 8)];
        assert!(test_board(
            white_pos,
            black_pos,
            (0, 5, 5, 0, 0, 0, 0, 0, 0, 4, 2, 0),
            (2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        ));
    }

    // nb_of catch/5 in a row/5 in a row can take/4 open/4 semi-open/4 close
    // 3 open/3 semi-open/3 close/2 open/2 semi-open/2 close

    // ⊕⊕⊕⊖_______________
    // ⊖⊖⊕⊖_______________
    // ___⊕_______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn test_infinite_loop_border() {
        let black_pos = vec![(0, 0), (0, 1), (0, 2), (1, 2), (2, 3)];
        let white_pos = vec![(1, 0), (1, 1), (0, 3), (1, 3)];
        assert!(test_board(
            white_pos,
            black_pos,
            (0, 0, 0, 0, 0, 0, 0, 3, 3, 0, 2, 0),
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4),
        ));
    }

    // _⊕_________________
    // _⊕_________________
    // __⊖________________
    // ___⊖_______________
    // ___________________
    // _____⊕_____________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn test_catch_2_main() {
        let black_pos = vec![(1, 1), (0, 1), (5, 5)];
        let white_pos = vec![(2, 2), (3, 3)];
        assert!(test_board(
            white_pos,
            black_pos,
            (2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0),
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2),
        ));
    }

    // _⊕_________________
    // _⊕_________________
    // __⊖________________
    // ___⊖_______________
    // ___________________
    // _____⊖_____________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn test_catch_2_error() {
        let black_pos = vec![(1, 1), (0, 1)];
        let white_pos = vec![(2, 2), (3, 3), (5, 5)];
        assert!(test_board(
            white_pos,
            black_pos,
            (2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0),
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0),
        ));
    }

    // ___________________
    // _⊕⊕⊖_______________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    // ___________________
    #[test]
    fn test_catch_2_wall() {
        let black_pos = vec![(1, 1), (1, 2)];
        let white_pos = vec![(1, 3)];
        assert!(test_board(
            white_pos,
            black_pos,
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2),
            (2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        ));
    }
}
