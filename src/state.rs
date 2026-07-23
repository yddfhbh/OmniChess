use crate::action::{Action, MoveAction};
use crate::board::{Board, BoardError};
use crate::movegen::is_pseudo_legal_move;
use crate::piece::{Color, Piece, PieceKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameError {
    GameAlreadyFinished,
    NoPieceAtSource,
    WrongSideToMove,
    CannotCaptureOwnPiece,
    IllegalMovement,
    PromotionNotImplemented,
    Board(BoardError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameResult {
    WhiteWin,
    BlackWin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    board: Board,
    side_to_move: Color,
    result: Option<GameResult>,
    ply: u32,
}

impl GameState {
    pub fn standard() -> Self {
        Self::from_board(Board::standard(), Color::White)
    }

    pub fn from_board(board: Board, side_to_move: Color) -> Self {
        Self {
            board,
            side_to_move,
            result: None,
            ply: 0,
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    pub fn result(&self) -> Option<GameResult> {
        self.result
    }

    pub fn ply(&self) -> u32 {
        self.ply
    }

    pub fn is_action_legal(&self, action: Action) -> Result<(), GameError> {
        if self.result.is_some() {
            return Err(GameError::GameAlreadyFinished);
        }

        match action {
            Action::Move(move_action) => self.validate_move(move_action),
        }
    }

    pub fn apply_action(&mut self, action: Action) -> Result<Option<Piece>, GameError> {
        self.is_action_legal(action)?;

        match action {
            Action::Move(move_action) => self.apply_move(move_action),
        }
    }

    fn validate_move(&self, action: MoveAction) -> Result<(), GameError> {
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

        if let Some(target_piece) = self.board.piece_at(action.to)
            && target_piece.color == moving_piece.color
        {
            return Err(GameError::CannotCaptureOwnPiece);
        }

        if !is_pseudo_legal_move(&self.board, action) {
            return Err(GameError::IllegalMovement);
        }

        Ok(())
    }

    fn apply_move(&mut self, action: MoveAction) -> Result<Option<Piece>, GameError> {
        let captured = self
            .board
            .move_piece(action.from, action.to)
            .map_err(GameError::Board)?;

        if let Some(piece) = &captured
            && piece.kind == PieceKind::King
        {
            self.result = Some(match piece.color {
                Color::White => GameResult::BlackWin,
                Color::Black => GameResult::WhiteWin,
            });
        }

        self.ply += 1;
        self.side_to_move = self.side_to_move.opposite();

        Ok(captured)
    }
}

#[cfg(test)]
mod tests {
    use super::{GameError, GameResult, GameState};
    use crate::action::Action;
    use crate::board::Board;
    use crate::piece::{Color, Piece, PieceKind};
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

        assert_eq!(game.side_to_move(), Color::White);
    }

    #[test]
    fn successful_move_changes_turn() {
        let mut game = GameState::standard();

        game.apply_action(action("e2e4")).unwrap();

        assert_eq!(game.side_to_move(), Color::Black);
        assert_eq!(game.ply(), 1);
        assert!(game.board().piece_at(square("e2")).is_none());
        let piece = game.board().piece_at(square("e4")).unwrap();
        assert_eq!(piece.color, Color::White);
        assert!(piece.has_moved);
    }

    #[test]
    fn rejects_black_move_on_white_turn() {
        let mut game = GameState::standard();

        let result = game.apply_action(action("e7e5"));

        assert_eq!(result, Err(GameError::WrongSideToMove));
        assert_eq!(game.side_to_move(), Color::White);
    }

    #[test]
    fn rejects_capturing_own_piece() {
        let mut game = GameState::standard();

        let result = game.apply_action(action("e1e2"));

        assert_eq!(result, Err(GameError::CannotCaptureOwnPiece));
        assert_eq!(game.side_to_move(), Color::White);
    }

    #[test]
    fn rejects_illegal_pawn_movement() {
        let mut game = GameState::standard();

        let result = game.apply_action(action("e2e5"));

        assert_eq!(result, Err(GameError::IllegalMovement));
        assert_eq!(game.side_to_move(), Color::White);
        assert!(game.board().piece_at(square("e2")).is_some());
        assert!(game.board().piece_at(square("e5")).is_none());
    }

    #[test]
    fn is_action_legal_rejects_empty_source_square() {
        let game = GameState::standard();

        assert_eq!(
            game.is_action_legal(action("e3e4")),
            Err(GameError::NoPieceAtSource)
        );
    }

    #[test]
    fn is_action_legal_rejects_capturing_own_piece() {
        let game = GameState::standard();

        assert_eq!(
            game.is_action_legal(action("d1d2")),
            Err(GameError::CannotCaptureOwnPiece)
        );
    }

    #[test]
    fn white_capturing_black_king_sets_white_win() {
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

        game.apply_action(action("e4e8")).unwrap();

        assert_eq!(game.result(), Some(GameResult::WhiteWin));
    }

    #[test]
    fn black_capturing_white_king_sets_black_win() {
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

        game.apply_action(action("e5e1")).unwrap();

        assert_eq!(game.result(), Some(GameResult::BlackWin));
    }

    #[test]
    fn finished_game_rejects_additional_actions() {
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

        game.apply_action(action("e4e8")).unwrap();
        let snapshot = game.clone();

        let result = game.apply_action(action("e8e7"));

        assert_eq!(result, Err(GameError::GameAlreadyFinished));
        assert_eq!(game, snapshot);
    }

    #[test]
    fn legal_move_increments_ply_by_one() {
        let mut game = GameState::standard();

        game.apply_action(action("e2e4")).unwrap();

        assert_eq!(game.ply(), 1);
    }

    #[test]
    fn illegal_move_attempt_does_not_change_state() {
        let mut game = GameState::standard();
        let snapshot = game.clone();

        let result = game.apply_action(action("e2e5"));

        assert_eq!(result, Err(GameError::IllegalMovement));
        assert_eq!(game, snapshot);
    }

    #[test]
    fn check_is_not_evaluated_for_moves_into_attacked_square() {
        let mut board = Board::empty();
        board.set_piece(
            square("e1"),
            Some(Piece::new(1, Color::White, PieceKind::King)),
        );
        board.set_piece(
            square("e8"),
            Some(Piece::new(2, Color::Black, PieceKind::Rook)),
        );
        let mut game = GameState::from_board(board, Color::White);

        let result = game.apply_action(action("e1e2"));

        assert!(result.is_ok());
    }

    #[test]
    fn main_gameplay_flows_through_apply_action() {
        let mut game = GameState::standard();

        game.apply_action(action("e2e4")).unwrap();

        assert!(game.board().piece_at(square("e4")).is_some());
    }
}
