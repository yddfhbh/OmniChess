use crate::action::{Action, MoveAction};
use crate::board::{Board, BoardError};
use crate::movegen::is_move_legal;
use crate::piece::{Color, Piece};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameError {
    NoPieceAtSource,
    WrongSideToMove,
    CannotCaptureOwnPiece,
    IllegalMovement,
    PromotionNotImplemented,
    Board(BoardError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    pub board: Board,
    pub side_to_move: Color,
}

impl GameState {
    pub fn standard() -> Self {
        Self {
            board: Board::standard(),
            side_to_move: Color::White,
        }
    }

    pub fn apply_action(&mut self, action: Action) -> Result<Option<Piece>, GameError> {
        match action {
            Action::Move(move_action) => self.apply_move(move_action),
        }
    }

    fn apply_move(&mut self, action: MoveAction) -> Result<Option<Piece>, GameError> {
        if action.promotion.is_some() {
            return Err(GameError::PromotionNotImplemented);
        }

        let moving_piece = self
            .board
            .piece_at(action.from)
            .ok_or(GameError::NoPieceAtSource)?;

        if moving_piece.color != self.side_to_move {
            return Err(GameError::WrongSideToMove);
        }

        if let Some(target_piece) = self.board.piece_at(action.to) {
            if target_piece.color == moving_piece.color {
                return Err(GameError::CannotCaptureOwnPiece);
            }
        }

        if !is_move_legal(&self.board, action) {
            return Err(GameError::IllegalMovement);
        }

        let captured = self
            .board
            .move_piece(action.from, action.to)
            .map_err(GameError::Board)?;

        self.side_to_move = self.side_to_move.opposite();

        Ok(captured)
    }
}

#[cfg(test)]
mod tests {
    use super::{GameError, GameState};
    use crate::action::Action;
    use crate::piece::Color;
    use crate::square::Square;

    fn action(value: &str) -> Action {
        Action::from_uci(value).unwrap()
    }

    fn square(value: &str) -> Square {
        Square::from_algebraic(value).unwrap()
    }

    #[test]
    fn game_starts_with_white() {
        let game = GameState::standard();

        assert_eq!(game.side_to_move, Color::White);
    }

    #[test]
    fn successful_move_changes_turn() {
        let mut game = GameState::standard();

        game.apply_action(action("e2e4")).unwrap();

        assert_eq!(game.side_to_move, Color::Black);
        assert!(game.board.piece_at(square("e2")).is_none());
        assert!(game.board.piece_at(square("e4")).is_some());
    }

    #[test]
    fn rejects_black_move_on_white_turn() {
        let mut game = GameState::standard();

        let result = game.apply_action(action("e7e5"));

        assert_eq!(result, Err(GameError::WrongSideToMove));
        assert_eq!(game.side_to_move, Color::White);
    }

    #[test]
    fn rejects_capturing_own_piece() {
        let mut game = GameState::standard();

        let result = game.apply_action(action("e1e2"));

        assert_eq!(result, Err(GameError::CannotCaptureOwnPiece));
        assert_eq!(game.side_to_move, Color::White);
    }

    #[test]
    fn rejects_illegal_pawn_movement() {
        let mut game = GameState::standard();

        let result = game.apply_action(action("e2e5"));

        assert_eq!(result, Err(GameError::IllegalMovement));
        assert_eq!(game.side_to_move, Color::White);
        assert!(game.board.piece_at(square("e2")).is_some());
        assert!(game.board.piece_at(square("e5")).is_none());
    }
}
