use crate::action::{Action, MoveAction};
use crate::board::{Board, BoardError};
use crate::constants::BOARD_SIZE;
use crate::movegen::is_pseudo_legal_move;
use crate::piece::{Color, Piece, PieceKind};
use crate::square::Square;
use std::fmt;

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

impl fmt::Display for GameResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WhiteWin => write!(formatter, "White wins"),
            Self::BlackWin => write!(formatter, "Black wins"),
        }
    }
}

impl fmt::Display for GameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GameAlreadyFinished => write!(formatter, "game is already finished"),
            Self::NoPieceAtSource => write!(formatter, "no piece on source square"),
            Self::WrongSideToMove => write!(formatter, "wrong side to move"),
            Self::CannotCaptureOwnPiece => write!(formatter, "cannot capture own piece"),
            Self::IllegalMovement => write!(formatter, "illegal movement"),
            Self::PromotionNotImplemented => write!(formatter, "promotion is not implemented"),
            Self::Board(BoardError::NoPieceAtSource(_)) => {
                write!(formatter, "no piece on source square")
            }
        }
    }
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

    pub fn legal_actions(&self) -> Vec<Action> {
        if self.result.is_some() {
            return Vec::new();
        }

        let mut actions = Vec::new();

        for rank in 0..BOARD_SIZE as u8 {
            for file in 0..BOARD_SIZE as u8 {
                let from = Square::new(file, rank).expect("board coordinates should be valid");
                actions.extend(self.legal_actions_from(from));
            }
        }

        actions
    }

    pub fn legal_actions_from(&self, from: Square) -> Vec<Action> {
        if self.result.is_some() {
            return Vec::new();
        }

        let Some(piece) = self.board.piece_at(from) else {
            return Vec::new();
        };

        if piece.color != self.side_to_move {
            return Vec::new();
        }

        let mut actions = Vec::new();

        for rank in 0..BOARD_SIZE as u8 {
            for file in 0..BOARD_SIZE as u8 {
                let to = Square::new(file, rank).expect("board coordinates should be valid");
                let action = Action::Move(MoveAction {
                    from,
                    to,
                    promotion: None,
                });

                if self.is_action_legal(action).is_ok() {
                    actions.push(action);
                }
            }
        }

        actions
    }

    pub fn apply_action(&mut self, action: Action) -> Result<Option<Piece>, GameError> {
        self.is_action_legal(action)?;

        match action {
            Action::Move(move_action) => self.apply_move(move_action),
        }
    }

    fn validate_move(&self, action: MoveAction) -> Result<(), GameError> {
        let moving_piece = self
            .board
            .piece_at(action.from)
            .ok_or(GameError::NoPieceAtSource)?;

        if action.promotion.is_some() || is_unimplemented_promotion_move(moving_piece.kind, action)
        {
            return Err(GameError::PromotionNotImplemented);
        }

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

fn is_unimplemented_promotion_move(kind: PieceKind, action: MoveAction) -> bool {
    kind == PieceKind::Pawn
        && ((action.to.rank() == BOARD_SIZE as u8 - 1 && action.from.rank() < action.to.rank())
            || (action.to.rank() == 0 && action.from.rank() > action.to.rank()))
}

#[cfg(test)]
mod tests {
    use super::{GameError, GameResult, GameState};
    use crate::action::{Action, MoveAction};
    use crate::board::Board;
    use crate::piece::{Color, Piece, PieceKind};
    use crate::square::Square;
    use std::collections::HashSet;

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

    #[test]
    fn start_position_has_twenty_legal_actions_for_white() {
        let game = GameState::standard();

        assert_eq!(game.legal_actions().len(), 20);
    }

    #[test]
    fn after_e2e4_black_has_twenty_legal_actions() {
        let mut game = GameState::standard();
        game.apply_action(action("e2e4")).unwrap();

        assert_eq!(game.legal_actions().len(), 20);
    }

    #[test]
    fn all_generated_actions_pass_is_action_legal() {
        let game = GameState::standard();

        for action in game.legal_actions() {
            assert!(game.is_action_legal(action).is_ok(), "{action}");
        }
    }

    #[test]
    fn generated_actions_do_not_capture_own_piece() {
        let game = GameState::standard();

        for action in game.legal_actions() {
            let Action::Move(move_action) = action;
            let moving_piece = game.board().piece_at(move_action.from).unwrap();

            if let Some(target_piece) = game.board().piece_at(move_action.to) {
                assert_ne!(moving_piece.color, target_piece.color);
            }
        }
    }

    #[test]
    fn generated_actions_are_unique() {
        let game = GameState::standard();
        let actions = game.legal_actions();
        let unique: HashSet<String> = actions.iter().map(ToString::to_string).collect();

        assert_eq!(actions.len(), unique.len());
    }

    #[test]
    fn repeated_legal_action_generation_is_deterministic() {
        let game = GameState::standard();

        assert_eq!(game.legal_actions(), game.legal_actions());
    }

    #[test]
    fn finished_game_has_no_legal_actions() {
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

        assert!(game.legal_actions().is_empty());
    }

    #[test]
    fn legal_actions_from_empty_square_is_empty() {
        let game = GameState::standard();

        assert!(game.legal_actions_from(square("e4")).is_empty());
    }

    #[test]
    fn legal_actions_from_opponent_square_is_empty() {
        let game = GameState::standard();

        assert!(game.legal_actions_from(square("e7")).is_empty());
    }

    #[test]
    fn legal_actions_from_e2_returns_only_e2e3_and_e2e4() {
        let game = GameState::standard();
        let actions: Vec<String> = game
            .legal_actions_from(square("e2"))
            .into_iter()
            .map(|action| action.to_string())
            .collect();

        assert_eq!(actions, vec!["e2e3", "e2e4"]);
    }

    #[test]
    fn legal_actions_from_b1_returns_only_b1a3_and_b1c3() {
        let game = GameState::standard();
        let actions: Vec<String> = game
            .legal_actions_from(square("b1"))
            .into_iter()
            .map(|action| action.to_string())
            .collect();

        assert_eq!(actions, vec!["b1a3", "b1c3"]);
    }

    #[test]
    fn grasshopper_generates_exact_jump_targets() {
        let mut board = Board::empty();
        board.set_piece(
            square("a1"),
            Some(Piece::new(1, Color::White, PieceKind::Grasshopper)),
        );
        board.set_piece(
            square("a3"),
            Some(Piece::new(2, Color::White, PieceKind::Pawn)),
        );
        board.set_piece(
            square("c3"),
            Some(Piece::new(3, Color::Black, PieceKind::Pawn)),
        );
        let game = GameState::from_board(board, Color::White);
        let actions: Vec<String> = game
            .legal_actions_from(square("a1"))
            .into_iter()
            .map(|action| action.to_string())
            .collect();

        assert_eq!(actions, vec!["a1a4", "a1d4"]);
    }

    #[test]
    fn grasshopper_without_hurdle_generates_no_actions() {
        let mut board = Board::empty();
        board.set_piece(
            square("a1"),
            Some(Piece::new(1, Color::White, PieceKind::Grasshopper)),
        );
        let game = GameState::from_board(board, Color::White);

        assert!(game.legal_actions_from(square("a1")).is_empty());
    }

    #[test]
    fn unimplemented_promotion_moves_are_not_generated() {
        let mut board = Board::empty();
        board.set_piece(
            square("a7"),
            Some(Piece::new(1, Color::White, PieceKind::Pawn)),
        );
        let game = GameState::from_board(board, Color::White);

        assert!(game.legal_actions_from(square("a7")).is_empty());
    }

    #[test]
    fn promotion_without_piece_selection_is_rejected() {
        let mut board = Board::empty();
        board.set_piece(
            square("a7"),
            Some(Piece::new(1, Color::White, PieceKind::Pawn)),
        );
        let game = GameState::from_board(board, Color::White);
        let action = Action::Move(MoveAction {
            from: square("a7"),
            to: square("a8"),
            promotion: None,
        });

        assert_eq!(
            game.is_action_legal(action),
            Err(GameError::PromotionNotImplemented)
        );
    }

    #[test]
    fn promotion_with_piece_selection_is_rejected() {
        let mut board = Board::empty();
        board.set_piece(
            square("a7"),
            Some(Piece::new(1, Color::White, PieceKind::Pawn)),
        );
        let game = GameState::from_board(board, Color::White);

        assert_eq!(
            game.is_action_legal(action("a7a8q")),
            Err(GameError::PromotionNotImplemented)
        );
    }
}
