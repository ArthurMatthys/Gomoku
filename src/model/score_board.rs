use super::super::render::board::SIZE_BOARD;

pub struct Score_board([[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD]);

impl Score_board{

    // Retrieve score_board[x][y]
    pub fn get(&self, x: usize, y:usize) -> Option<[(u8, Option<bool>, Option<bool>); 4]>{
        self.0.get(x).map(|b| b.get(y)).flatten().cloned()
    }

    // Print score_board
    pub fn print(&self) -> () {
        self.0.iter().for_each(|x| {
            x.iter().for_each(|y| {
                y.iter().for_each(|el|
                    print!("{:2}", el.0));
                print!("||");}
                );
            println!();
        })
    }

}
