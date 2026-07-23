use crate::constants::BOARD_SIZE;
use crate::piece::{Color, PieceKind};
use crate::square::Square;
use crate::state::{GameResult, GameState};

/// Score used for terminal wins.
pub const MATE_SCORE: i32 = 1_000_000;

/// Large score used as search infinity.
pub const INFINITY_SCORE: i32 = 2_000_000;

/// Returns the material value for a piece kind.
pub const fn piece_value(kind: PieceKind) -> i32 {
    match kind {
        PieceKind::Pawn => 100,
        PieceKind::Knight => 320,
        PieceKind::Bishop => 330,
        PieceKind::Rook => 500,
        PieceKind::Queen => 900,
        PieceKind::Grasshopper => 350,
        PieceKind::King => 20_000,
    }
}

/// Evaluates the current state from one side's perspective.
pub fn evaluate_for(state: &GameState, perspective: Color) -> i32 {
    if let Some(result) = state.result() {
        return match (result, perspective) {
            (GameResult::WhiteWin, Color::White) | (GameResult::BlackWin, Color::Black) => {
                MATE_SCORE
            }
            (GameResult::WhiteWin, Color::Black) | (GameResult::BlackWin, Color::White) => {
                -MATE_SCORE
            }
        };
    }

    let mut material_score = 0;

    for rank in 0..BOARD_SIZE as u8 {
        for file in 0..BOARD_SIZE as u8 {
            let Some(square) = Square::new(file, rank) else {
                continue;
            };

            if let Some(piece) = state.board().piece_at(square) {
                let value = piece_value(piece.kind);
                if piece.color == perspective {
                    material_score += value;
                } else {
                    material_score -= value;
                }
            }
        }
    }

    let our_mobility = state.legal_actions_for(perspective).len() as i32;
    let their_mobility = state.legal_actions_for(perspective.opposite()).len() as i32;
    let mobility_score = (our_mobility - their_mobility) * 2;

    material_score + mobility_score
}

#[cfg(test)]
mod tests {
    use super::{MATE_SCORE, evaluate_for, piece_value};
    use crate::action::Action;
    use crate::board::Board;
    use crate::piece::{Color, Piece, PieceKind};
    use crate::square::Square;
    use crate::state::GameState;

    fn square(value: &str) -> Square {
        Square::from_algebraic(value).unwrap()
    }

    #[test]
    fn start_position_scores_are_opposites() {
        let game = GameState::standard();

        assert_eq!(
            evaluate_for(&game, Color::White),
            -evaluate_for(&game, Color::Black)
        );
    }

    #[test]
    fn equal_material_and_mobility_scores_zero() {
        let mut board = Board::empty();
        board.set_piece(
            square("d4"),
            Some(Piece::new(1, Color::White, PieceKind::King)),
        );
        board.set_piece(
            square("d5"),
            Some(Piece::new(2, Color::Black, PieceKind::King)),
        );
        let game = GameState::from_board(board, Color::White);

        assert_eq!(evaluate_for(&game, Color::White), 0);
    }

    #[test]
    fn extra_white_queen_is_positive_for_white() {
        let mut board = Board::empty();
        board.set_piece(
            square("a1"),
            Some(Piece::new(1, Color::White, PieceKind::King)),
        );
        board.set_piece(
            square("h8"),
            Some(Piece::new(2, Color::Black, PieceKind::King)),
        );
        board.set_piece(
            square("d4"),
            Some(Piece::new(3, Color::White, PieceKind::Queen)),
        );
        let game = GameState::from_board(board, Color::White);

        assert!(evaluate_for(&game, Color::White) > 0);
    }

    #[test]
    fn extra_black_rook_is_negative_for_white() {
        let mut board = Board::empty();
        board.set_piece(
            square("a1"),
            Some(Piece::new(1, Color::White, PieceKind::King)),
        );
        board.set_piece(
            square("h8"),
            Some(Piece::new(2, Color::Black, PieceKind::King)),
        );
        board.set_piece(
            square("d4"),
            Some(Piece::new(3, Color::Black, PieceKind::Rook)),
        );
        let game = GameState::from_board(board, Color::White);

        assert!(evaluate_for(&game, Color::White) < 0);
    }

    #[test]
    fn white_win_scores_as_mate() {
        let mut board = Board::empty();
        board.set_piece(
            square("e4"),
            Some(Piece::new(1, Color::White, PieceKind::Queen)),
        );
        board.set_piece(
            square("e8"),
            Some(Piece::new(2, Color::Black, PieceKind::King)),
        );
        let mut game = GameState::from_board(board, Color::White);
        game.apply_action(Action::from_uci("e4e8").unwrap())
            .unwrap();

        assert_eq!(evaluate_for(&game, Color::White), MATE_SCORE);
        assert_eq!(evaluate_for(&game, Color::Black), -MATE_SCORE);
    }

    #[test]
    fn black_win_scores_as_mate() {
        let mut board = Board::empty();
        board.set_piece(
            square("e5"),
            Some(Piece::new(1, Color::Black, PieceKind::Queen)),
        );
        board.set_piece(
            square("e1"),
            Some(Piece::new(2, Color::White, PieceKind::King)),
        );
        let mut game = GameState::from_board(board, Color::Black);
        game.apply_action(Action::from_uci("e5e1").unwrap())
            .unwrap();

        assert_eq!(evaluate_for(&game, Color::White), -MATE_SCORE);
        assert_eq!(evaluate_for(&game, Color::Black), MATE_SCORE);
    }

    #[test]
    fn evaluation_does_not_modify_state() {
        let game = GameState::standard();
        let snapshot = game.clone();

        let _ = evaluate_for(&game, Color::White);

        assert_eq!(game, snapshot);
    }

    #[test]
    fn piece_values_match_spec() {
        assert_eq!(piece_value(PieceKind::Pawn), 100);
        assert_eq!(piece_value(PieceKind::Knight), 320);
        assert_eq!(piece_value(PieceKind::Bishop), 330);
        assert_eq!(piece_value(PieceKind::Rook), 500);
        assert_eq!(piece_value(PieceKind::Queen), 900);
        assert_eq!(piece_value(PieceKind::Grasshopper), 350);
        assert_eq!(piece_value(PieceKind::King), 20_000);
    }
}
