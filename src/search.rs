use crate::action::Action;
use crate::evaluate::{INFINITY_SCORE, MATE_SCORE, evaluate_for, piece_value};
use crate::piece::PieceKind;
use crate::state::{GameResult, GameState};

/// Result of a fixed-depth search.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchResult {
    pub best_action: Option<Action>,
    pub score: i32,
    pub depth: u8,
    pub nodes: u64,
}

/// Searches for the best action from the current position.
pub fn search_best_action(state: &GameState, depth: u8) -> SearchResult {
    if depth == 0 {
        return SearchResult {
            best_action: None,
            score: evaluate_for(state, state.side_to_move()),
            depth: 0,
            nodes: 1,
        };
    }

    let mut nodes = 0;
    let mut actions = state.legal_actions();

    if actions.is_empty() {
        nodes += 1;
        return SearchResult {
            best_action: None,
            score: terminal_or_static_score(state, 0),
            depth,
            nodes,
        };
    }

    order_actions(state, &mut actions);

    let mut best_action = None;
    let mut best_score = -INFINITY_SCORE;
    let mut alpha = -INFINITY_SCORE;
    let beta = INFINITY_SCORE;

    for action in actions {
        let mut next_state = state.clone();
        if let Err(error) = next_state.apply_action(action) {
            panic!("legal action {action} failed during search: {error}");
        }

        let score = -negamax(&next_state, depth - 1, -beta, -alpha, 1, &mut nodes);
        if score > best_score {
            best_score = score;
            best_action = Some(action);
        }
        if score > alpha {
            alpha = score;
        }
    }

    SearchResult {
        best_action,
        score: best_score,
        depth,
        nodes,
    }
}

fn negamax(
    state: &GameState,
    depth: u8,
    mut alpha: i32,
    beta: i32,
    distance_from_root: i32,
    nodes: &mut u64,
) -> i32 {
    *nodes += 1;

    if state.result().is_some() {
        return terminal_or_static_score(state, distance_from_root);
    }

    if depth == 0 {
        return evaluate_for(state, state.side_to_move());
    }

    let mut actions = state.legal_actions();
    if actions.is_empty() {
        return evaluate_for(state, state.side_to_move());
    }

    order_actions(state, &mut actions);

    let mut best_score = -INFINITY_SCORE;

    for action in actions {
        let mut next_state = state.clone();
        if let Err(error) = next_state.apply_action(action) {
            panic!("legal action {action} failed during search: {error}");
        }

        let score = -negamax(
            &next_state,
            depth - 1,
            -beta,
            -alpha,
            distance_from_root + 1,
            nodes,
        );

        if score > best_score {
            best_score = score;
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            break;
        }
    }

    best_score
}

fn terminal_or_static_score(state: &GameState, distance_from_root: i32) -> i32 {
    match state.result() {
        Some(GameResult::WhiteWin) => {
            if state.side_to_move() == crate::piece::Color::White {
                MATE_SCORE - distance_from_root
            } else {
                -MATE_SCORE + distance_from_root
            }
        }
        Some(GameResult::BlackWin) => {
            if state.side_to_move() == crate::piece::Color::Black {
                MATE_SCORE - distance_from_root
            } else {
                -MATE_SCORE + distance_from_root
            }
        }
        None => evaluate_for(state, state.side_to_move()),
    }
}

fn order_actions(state: &GameState, actions: &mut [Action]) {
    actions.sort_by_key(|action| match action {
        Action::Move(move_action) => match state.board().piece_at(move_action.to) {
            Some(piece) if piece.kind == PieceKind::King => (0, -piece_value(piece.kind)),
            Some(piece) => (1, -piece_value(piece.kind)),
            None => (2, 0),
        },
    });
}

#[cfg(test)]
mod tests {
    use super::{order_actions, search_best_action};
    use crate::action::Action;
    use crate::board::Board;
    use crate::evaluate::{MATE_SCORE, evaluate_for};
    use crate::piece::{Color, Piece, PieceKind};
    use crate::search::SearchResult;
    use crate::square::Square;
    use crate::state::GameState;

    fn square(value: &str) -> Square {
        Square::from_algebraic(value).unwrap()
    }

    fn action(value: &str) -> Action {
        Action::from_uci(value).unwrap()
    }

    #[test]
    fn depth_zero_returns_static_evaluation_only() {
        let game = GameState::standard();
        let result = search_best_action(&game, 0);

        assert_eq!(
            result,
            SearchResult {
                best_action: None,
                score: evaluate_for(&game, Color::White),
                depth: 0,
                nodes: 1,
            }
        );
    }

    #[test]
    fn depth_one_returns_legal_action_from_start() {
        let game = GameState::standard();
        let result = search_best_action(&game, 1);

        assert!(result.best_action.is_some());
    }

    #[test]
    fn returned_best_action_is_legal() {
        let game = GameState::standard();
        let result = search_best_action(&game, 1);
        let best_action = result.best_action.unwrap();

        assert!(game.is_action_legal(best_action).is_ok());
    }

    #[test]
    fn repeated_search_is_deterministic() {
        let game = GameState::standard();

        assert_eq!(search_best_action(&game, 2), search_best_action(&game, 2));
    }

    #[test]
    fn search_does_not_modify_original_state() {
        let game = GameState::standard();
        let snapshot = game.clone();

        let _ = search_best_action(&game, 2);

        assert_eq!(game, snapshot);
    }

    #[test]
    fn immediate_king_capture_is_selected() {
        let mut board = Board::empty();
        board.set_piece(
            square("e4"),
            Some(Piece::new(1, Color::White, PieceKind::Queen)),
        );
        board.set_piece(
            square("e8"),
            Some(Piece::new(2, Color::Black, PieceKind::King)),
        );
        board.set_piece(
            square("a8"),
            Some(Piece::new(3, Color::Black, PieceKind::Rook)),
        );
        let game = GameState::from_board(board, Color::White);

        let result = search_best_action(&game, 1);

        assert_eq!(result.best_action, Some(action("e4e8")));
    }

    #[test]
    fn mate_is_preferred_over_non_mating_capture() {
        let mut board = Board::empty();
        board.set_piece(
            square("e4"),
            Some(Piece::new(1, Color::White, PieceKind::Queen)),
        );
        board.set_piece(
            square("e8"),
            Some(Piece::new(2, Color::Black, PieceKind::King)),
        );
        board.set_piece(
            square("a8"),
            Some(Piece::new(3, Color::Black, PieceKind::Queen)),
        );
        let game = GameState::from_board(board, Color::White);

        let result = search_best_action(&game, 1);

        assert_eq!(result.best_action, Some(action("e4e8")));
        assert!(result.score >= MATE_SCORE - 1);
    }

    #[test]
    fn search_avoids_forced_king_loss_when_possible() {
        let mut board = Board::empty();
        board.set_piece(
            square("a1"),
            Some(Piece::new(1, Color::White, PieceKind::King)),
        );
        board.set_piece(
            square("b1"),
            Some(Piece::new(2, Color::White, PieceKind::Rook)),
        );
        board.set_piece(
            square("a2"),
            Some(Piece::new(3, Color::Black, PieceKind::Queen)),
        );
        board.set_piece(
            square("h8"),
            Some(Piece::new(4, Color::Black, PieceKind::King)),
        );
        let game = GameState::from_board(board, Color::White);

        let result = search_best_action(&game, 2);

        assert_eq!(result.best_action, Some(action("a1a2")));
    }

    #[test]
    fn faster_mate_is_preferred_to_slower_lines() {
        let mut board = Board::empty();
        board.set_piece(
            square("e4"),
            Some(Piece::new(1, Color::White, PieceKind::Queen)),
        );
        board.set_piece(
            square("e8"),
            Some(Piece::new(2, Color::Black, PieceKind::King)),
        );
        board.set_piece(
            square("a8"),
            Some(Piece::new(3, Color::Black, PieceKind::Rook)),
        );
        board.set_piece(
            square("h1"),
            Some(Piece::new(4, Color::White, PieceKind::Rook)),
        );
        let game = GameState::from_board(board, Color::White);

        let result = search_best_action(&game, 2);

        assert_eq!(result.best_action, Some(action("e4e8")));
    }

    #[test]
    fn finished_game_has_no_best_action() {
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

        assert_eq!(search_best_action(&game, 1).best_action, None);
    }

    #[test]
    fn no_legal_actions_position_returns_none_without_panicking() {
        let mut board = Board::empty();
        board.set_piece(
            square("a8"),
            Some(Piece::new(1, Color::White, PieceKind::Pawn)),
        );
        board.set_piece(
            square("h8"),
            Some(Piece::new(2, Color::Black, PieceKind::King)),
        );
        let game = GameState::from_board(board, Color::White);

        assert_eq!(search_best_action(&game, 2).best_action, None);
    }

    #[test]
    fn node_count_is_non_zero() {
        let game = GameState::standard();

        assert!(search_best_action(&game, 1).nodes > 0);
    }

    #[test]
    fn deeper_search_does_not_reduce_nodes_in_quiet_position() {
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
            Some(Piece::new(3, Color::White, PieceKind::Rook)),
        );
        let game = GameState::from_board(board, Color::White);

        let depth_one = search_best_action(&game, 1);
        let depth_two = search_best_action(&game, 2);

        assert!(depth_two.nodes >= depth_one.nodes);
    }

    #[test]
    fn order_places_king_capture_first() {
        let mut board = Board::empty();
        board.set_piece(
            square("e4"),
            Some(Piece::new(1, Color::White, PieceKind::Queen)),
        );
        board.set_piece(
            square("e8"),
            Some(Piece::new(2, Color::Black, PieceKind::King)),
        );
        board.set_piece(
            square("a8"),
            Some(Piece::new(3, Color::Black, PieceKind::Rook)),
        );
        let game = GameState::from_board(board, Color::White);
        let mut actions = vec![action("e4a8"), action("e4e8"), action("e4e5")];

        order_actions(&game, &mut actions);

        assert_eq!(actions[0], action("e4e8"));
    }

    #[test]
    fn order_places_higher_value_capture_before_lower_value_capture() {
        let mut board = Board::empty();
        board.set_piece(
            square("d4"),
            Some(Piece::new(1, Color::White, PieceKind::Queen)),
        );
        board.set_piece(
            square("d8"),
            Some(Piece::new(2, Color::Black, PieceKind::Rook)),
        );
        board.set_piece(
            square("h4"),
            Some(Piece::new(3, Color::Black, PieceKind::Pawn)),
        );
        board.set_piece(
            square("a1"),
            Some(Piece::new(4, Color::White, PieceKind::King)),
        );
        board.set_piece(
            square("h8"),
            Some(Piece::new(5, Color::Black, PieceKind::King)),
        );
        let game = GameState::from_board(board, Color::White);
        let mut actions = vec![action("d4h4"), action("d4d8")];

        order_actions(&game, &mut actions);

        assert_eq!(actions, vec![action("d4d8"), action("d4h4")]);
    }

    #[test]
    fn order_preserves_original_order_for_ties() {
        let mut board = Board::empty();
        board.set_piece(
            square("d4"),
            Some(Piece::new(1, Color::White, PieceKind::Queen)),
        );
        board.set_piece(
            square("a1"),
            Some(Piece::new(2, Color::White, PieceKind::King)),
        );
        board.set_piece(
            square("h8"),
            Some(Piece::new(3, Color::Black, PieceKind::King)),
        );
        let game = GameState::from_board(board, Color::White);
        let mut actions = vec![action("d4d5"), action("d4d6"), action("d4d7")];

        order_actions(&game, &mut actions);

        assert_eq!(
            actions,
            vec![action("d4d5"), action("d4d6"), action("d4d7")]
        );
    }
}
