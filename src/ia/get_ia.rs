use super::super::model::point;
use rand::distributions::{Distribution, Uniform};
use super::super::model::game;

pub fn get_ia(game: &mut game::Game) -> usize {
    // copy get_ia
    let mut rng = rand::thread_rng();
    let choice = Uniform::from(0..20);
    point::index_of_coord(choice.sample(&mut rng), choice.sample(&mut rng))   
}
