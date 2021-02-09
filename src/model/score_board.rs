use super::super::render::board::SIZE_BOARD;

pub struct ScoreBoard([[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD]);

impl ScoreBoard{
    
    // Retrieve score_board[x][y]
    pub fn get(&self, x: usize, y:usize, dir:usize) -> (u8, Option<bool>, Option<bool>){
        self.0[x][y][dir]
    }

    // Retrieve score_board[x][y]
    pub fn get_check(&self, x: usize, y:usize, dir:usize) -> Option<(u8, Option<bool>, Option<bool>)>{
        self.0.get(x).map(|b| b.get(y).map(|c| c.get(dir))).flatten().flatten().cloned()
    }

    pub fn set(&mut self, x: usize, y:usize, dir:usize, score: (u8, Option<bool>, Option<bool>)) -> () {
        self.0[x][y][dir] = score;
    }

    pub fn reset(&mut self, x: usize, y:usize, dir:usize) -> () {
        self.0[x][y][dir] = (0, Some(false), Some(false));
    }
    //pub fn get(&self, x: usize, y:usize) -> Option<[(u8, Option<bool>, Option<bool>); 4]>{
    //    self.0.get(x).map(|b| b.get(y)).flatten().cloned()
    //}

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

impl From<[[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD]> for ScoreBoard {
    fn from(item: [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD]) -> Self{
        ScoreBoard(item)
    }
}

impl From<ScoreBoard> for [[[(u8, Option<bool>, Option<bool>); 4]; SIZE_BOARD]; SIZE_BOARD]{
    fn from(item: ScoreBoard) -> Self{
        item.0
    }
}
