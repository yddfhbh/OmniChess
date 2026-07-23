mod action;
mod board;
mod movegen;
mod piece;
mod square;
mod state;

use action::Action;
use board::Board;
use movegen::is_move_legal;
use piece::{Color, Piece, PieceKind};
use square::Square;
use state::GameState;

fn square(value: &str) -> Square {
    Square::from_algebraic(value).expect("올바른 체스 좌표여야 합니다.")
}

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

    let mut demo = Board::empty();

    demo.set_piece(
        square("a1"),
        Some(Piece::new(Color::White, PieceKind::Grasshopper)),
    );

    demo.set_piece(
        square("a3"),
        Some(Piece::new(Color::White, PieceKind::Pawn)),
    );

    demo.set_piece(
        square("a4"),
        Some(Piece::new(Color::Black, PieceKind::Queen)),
    );

    println!("\n그래스호퍼 데모:");
    demo.print();

    let grasshopper_move = action::MoveAction::from_uci("a1a4").expect("올바른 이동이어야 합니다.");

    println!("a1a4 이동 가능: {}", is_move_legal(&demo, grasshopper_move));
}
