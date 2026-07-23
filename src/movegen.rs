use crate::action::MoveAction;
use crate::board::Board;
use crate::piece::{Color, PieceKind};
use crate::square::Square;

pub fn is_move_legal(board: &Board, action: MoveAction) -> bool {
    let Some(piece) = board.piece_at(action.from) else {
        return false;
    };

    if action.from == action.to {
        return false;
    }

    match piece.kind {
        PieceKind::Pawn => is_pawn_move_legal(board, action),
        PieceKind::Knight => is_knight_move_legal(action),
        PieceKind::Bishop => is_bishop_move_legal(board, action),
        PieceKind::Rook => is_rook_move_legal(board, action),
        PieceKind::Queen => is_queen_move_legal(board, action),
        PieceKind::King => is_king_move_legal(action),

        // 다음 단계에서 구현
        PieceKind::Grasshopper => false,
    }
}

fn is_pawn_move_legal(board: &Board, action: MoveAction) -> bool {
    let piece = board
        .piece_at(action.from)
        .expect("출발 칸에 폰이 있어야 합니다.");

    let file_delta = i16::from(action.to.file()) - i16::from(action.from.file());
    let rank_delta = i16::from(action.to.rank()) - i16::from(action.from.rank());

    let (direction, starting_rank) = match piece.color {
        Color::White => (1_i16, 1_u8),
        Color::Black => (-1_i16, 6_u8),
    };

    let target_piece = board.piece_at(action.to);

    // 같은 파일로 전진
    if file_delta == 0 {
        if target_piece.is_some() {
            return false;
        }

        // 한 칸 전진
        if rank_delta == direction {
            return true;
        }

        // 시작 위치에서 두 칸 전진
        if action.from.rank() == starting_rank && rank_delta == direction * 2 {
            let middle_rank = i16::from(action.from.rank()) + direction;

            let middle_square = Square::new(action.from.file(), middle_rank as u8)
                .expect("중간 칸은 보드 안이어야 합니다.");

            return board.piece_at(middle_square).is_none();
        }

        return false;
    }

    // 대각선 한 칸의 상대 기물 포획
    if file_delta.abs() == 1 && rank_delta == direction {
        return matches!(
            target_piece,
            Some(target) if target.color != piece.color
        );
    }

    false
}

fn is_knight_move_legal(action: MoveAction) -> bool {
    let file_delta = (i16::from(action.to.file()) - i16::from(action.from.file())).abs();

    let rank_delta = (i16::from(action.to.rank()) - i16::from(action.from.rank())).abs();

    matches!((file_delta, rank_delta), (1, 2) | (2, 1))
}

fn is_bishop_move_legal(board: &Board, action: MoveAction) -> bool {
    let file_delta = i16::from(action.to.file()) - i16::from(action.from.file());

    let rank_delta = i16::from(action.to.rank()) - i16::from(action.from.rank());

    if file_delta.abs() != rank_delta.abs() {
        return false;
    }

    is_path_clear(board, action)
}

fn is_rook_move_legal(board: &Board, action: MoveAction) -> bool {
    let same_file = action.from.file() == action.to.file();
    let same_rank = action.from.rank() == action.to.rank();

    if !same_file && !same_rank {
        return false;
    }

    is_path_clear(board, action)
}

fn is_queen_move_legal(board: &Board, action: MoveAction) -> bool {
    let file_delta = i16::from(action.to.file()) - i16::from(action.from.file());

    let rank_delta = i16::from(action.to.rank()) - i16::from(action.from.rank());

    let diagonal = file_delta.abs() == rank_delta.abs();
    let straight = file_delta == 0 || rank_delta == 0;

    if !diagonal && !straight {
        return false;
    }

    is_path_clear(board, action)
}

fn is_king_move_legal(action: MoveAction) -> bool {
    let file_delta = (i16::from(action.to.file()) - i16::from(action.from.file())).abs();

    let rank_delta = (i16::from(action.to.rank()) - i16::from(action.from.rank())).abs();

    file_delta <= 1 && rank_delta <= 1 && (file_delta != 0 || rank_delta != 0)
}

fn is_path_clear(board: &Board, action: MoveAction) -> bool {
    let from_file = i16::from(action.from.file());
    let from_rank = i16::from(action.from.rank());
    let to_file = i16::from(action.to.file());
    let to_rank = i16::from(action.to.rank());

    let file_step = (to_file - from_file).signum();
    let rank_step = (to_rank - from_rank).signum();

    let mut file = from_file + file_step;
    let mut rank = from_rank + rank_step;

    // 도착 칸은 검사하지 않는다.
    // 상대 기물 포획과 자기 기물 포획 여부는 GameState에서 처리한다.
    while file != to_file || rank != to_rank {
        let square =
            Square::new(file as u8, rank as u8).expect("이동 경로는 보드 안이어야 합니다.");

        if board.piece_at(square).is_some() {
            return false;
        }

        file += file_step;
        rank += rank_step;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::is_move_legal;
    use crate::action::MoveAction;
    use crate::board::Board;
    use crate::piece::{Color, Piece, PieceKind};
    use crate::square::Square;

    fn move_action(value: &str) -> MoveAction {
        MoveAction::from_uci(value).unwrap()
    }

    fn square(value: &str) -> Square {
        Square::from_algebraic(value).unwrap()
    }

    fn place(board: &mut Board, position: &str, color: Color, kind: PieceKind) {
        board.set_piece(square(position), Some(Piece::new(color, kind)));
    }

    #[test]
    fn white_pawn_moves_one_square_forward() {
        let board = Board::standard();

        assert!(is_move_legal(&board, move_action("e2e3")));
    }

    #[test]
    fn white_pawn_moves_two_squares_from_start() {
        let board = Board::standard();

        assert!(is_move_legal(&board, move_action("e2e4")));
    }

    #[test]
    fn pawn_cannot_move_three_squares() {
        let board = Board::standard();

        assert!(!is_move_legal(&board, move_action("e2e5")));
    }

    #[test]
    fn pawn_cannot_jump_over_piece() {
        let mut board = Board::standard();

        place(&mut board, "e3", Color::Black, PieceKind::Knight);

        assert!(!is_move_legal(&board, move_action("e2e4")));
    }

    #[test]
    fn pawn_captures_enemy_diagonally() {
        let mut board = Board::standard();

        place(&mut board, "d3", Color::Black, PieceKind::Knight);

        assert!(is_move_legal(&board, move_action("e2d3")));
    }

    #[test]
    fn pawn_cannot_move_diagonally_without_capture() {
        let board = Board::standard();

        assert!(!is_move_legal(&board, move_action("e2d3")));
    }

    #[test]
    fn black_pawn_moves_toward_lower_ranks() {
        let board = Board::standard();

        assert!(is_move_legal(&board, move_action("e7e5")));
        assert!(!is_move_legal(&board, move_action("e7e4")));
    }

    #[test]
    fn knight_moves_two_files_and_one_rank() {
        let board = Board::standard();

        assert!(is_move_legal(&board, move_action("b1d2")));
    }

    #[test]
    fn knight_moves_one_file_and_two_ranks() {
        let board = Board::standard();

        assert!(is_move_legal(&board, move_action("b1c3")));
    }

    #[test]
    fn knight_can_jump_over_pieces() {
        let board = Board::standard();

        assert!(is_move_legal(&board, move_action("b1c3")));
    }

    #[test]
    fn knight_cannot_move_straight() {
        let board = Board::standard();

        assert!(!is_move_legal(&board, move_action("b1b3")));
    }

    #[test]
    fn black_knight_uses_same_movement() {
        let board = Board::standard();

        assert!(is_move_legal(&board, move_action("g8f6")));
    }

    #[test]
    fn bishop_moves_diagonally() {
        let mut board = Board::empty();

        place(&mut board, "c1", Color::White, PieceKind::Bishop);

        assert!(is_move_legal(&board, move_action("c1h6")));
    }

    #[test]
    fn bishop_cannot_move_straight() {
        let mut board = Board::empty();

        place(&mut board, "c1", Color::White, PieceKind::Bishop);

        assert!(!is_move_legal(&board, move_action("c1c5")));
    }

    #[test]
    fn bishop_cannot_jump_over_piece() {
        let mut board = Board::empty();

        place(&mut board, "c1", Color::White, PieceKind::Bishop);

        place(&mut board, "e3", Color::Black, PieceKind::Pawn);

        assert!(!is_move_legal(&board, move_action("c1h6")));
    }

    #[test]
    fn rook_moves_straight() {
        let mut board = Board::empty();

        place(&mut board, "a1", Color::White, PieceKind::Rook);

        assert!(is_move_legal(&board, move_action("a1a8")));
        assert!(is_move_legal(&board, move_action("a1h1")));
    }

    #[test]
    fn rook_cannot_move_diagonally() {
        let mut board = Board::empty();

        place(&mut board, "a1", Color::White, PieceKind::Rook);

        assert!(!is_move_legal(&board, move_action("a1d4")));
    }

    #[test]
    fn rook_cannot_jump_over_piece() {
        let mut board = Board::empty();

        place(&mut board, "a1", Color::White, PieceKind::Rook);

        place(&mut board, "a4", Color::Black, PieceKind::Pawn);

        assert!(!is_move_legal(&board, move_action("a1a8")));
    }

    #[test]
    fn queen_moves_diagonally_and_straight() {
        let mut board = Board::empty();

        place(&mut board, "d4", Color::White, PieceKind::Queen);

        assert!(is_move_legal(&board, move_action("d4h8")));
        assert!(is_move_legal(&board, move_action("d4d8")));
        assert!(is_move_legal(&board, move_action("d4a4")));
    }

    #[test]
    fn queen_rejects_invalid_movement_shape() {
        let mut board = Board::empty();

        place(&mut board, "d4", Color::White, PieceKind::Queen);

        assert!(!is_move_legal(&board, move_action("d4f5")));
    }

    #[test]
    fn piece_cannot_remain_on_same_square() {
        let board = Board::standard();

        assert!(!is_move_legal(&board, move_action("b1b1")));
    }

    #[test]
    fn king_moves_one_square_in_any_direction() {
        let mut board = Board::empty();

        place(&mut board, "e4", Color::White, PieceKind::King);

        assert!(is_move_legal(&board, move_action("e4e5")));
        assert!(is_move_legal(&board, move_action("e4f5")));
        assert!(is_move_legal(&board, move_action("e4f4")));
        assert!(is_move_legal(&board, move_action("e4d3")));
    }

    #[test]
    fn king_cannot_move_more_than_one_square() {
        let mut board = Board::empty();

        place(&mut board, "e4", Color::White, PieceKind::King);

        assert!(!is_move_legal(&board, move_action("e4e6")));
        assert!(!is_move_legal(&board, move_action("e4g4")));
        assert!(!is_move_legal(&board, move_action("e4g6")));
    }

    #[test]
    fn king_cannot_move_like_knight() {
        let mut board = Board::empty();

        place(&mut board, "e4", Color::White, PieceKind::King);

        assert!(!is_move_legal(&board, move_action("e4f6")));
    }
}
