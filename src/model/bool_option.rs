/// return the opponent player pawn
pub fn get_opp(pawn: Option<bool>) -> Option<bool> {
    pawn.map(|x| !x)
}
