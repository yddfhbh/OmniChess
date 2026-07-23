use omnichess::state::GameState;

fn main() {
    println!("OmniChess");

    let game = GameState::standard();
    game.board().print();
}
