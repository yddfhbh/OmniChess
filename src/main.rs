mod action;
mod board;
mod movegen;
mod piece;
mod square;
mod state;

use action::Action;
use state::GameState;

fn main() {
    println!("OmniChess");

    let mut game = GameState::standard();
    game.board.print();

    let action = Action::from_uci("e2e4").expect("올바른 UCI 형식이어야 합니다.");

    game.apply_action(action)
        .expect("현재 게임 상태에서 적용 가능한 행동이어야 합니다.");

    println!("e2e4 이동 후:");
    game.board.print();

    println!("다음 차례: {:?}", game.side_to_move);
}
