use super::super::render::board::SIZE_BOARD;

#[derive(Clone, Copy)]
pub struct Board([[Option<bool>; SIZE_BOARD]; SIZE_BOARD]);


impl Board {


    // Retrieve board[x][y]
    pub fn get(&self, x: usize, y:usize) -> Option<Option<bool>>{
        self.0.get(x).map(|b| b.get(y)).flatten().cloned()
    }

    // Retrieve value of board[x][y], don't care about None
    pub fn get_pawn(&self, x: usize, y:usize) -> Option<bool>{
        self.0.get(x).map(|b| b.get(y)).flatten().cloned().flatten()
    }

    pub fn set(&mut self, x: usize, y:usize, pawn: Option<bool>) -> (){
        self.0[x][y] = pawn;
    }

    // Print board
    pub fn print(&self) -> ()
    {
        self.0.iter().for_each(|x| {
            x.iter().for_each(|el|
                match el {
                    Some(true) => print!("⊖"),
                    Some(false) => print!("⊕"),
                    None => print!("_"),
                });
            println!(); })
    }
}

impl From<[[Option<bool>; SIZE_BOARD]; SIZE_BOARD]> for Board{
    fn from(item: [[Option<bool>; SIZE_BOARD]; SIZE_BOARD]) -> Self{
        Board(item)
    }
}
impl From<Board> for [[Option<bool>; SIZE_BOARD]; SIZE_BOARD]{
    fn from(item: Board) -> Self{
        item.0
    }
}
