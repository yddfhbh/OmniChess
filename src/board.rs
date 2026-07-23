use std::array::from_fn;
use std::fmt;

use crate::constants::BOARD_SIZE;
use crate::piece::{Color, Piece, PieceKind};
use crate::square::Square;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardError {
    NoPieceAtSource(Square),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    squares: [[Option<Piece>; BOARD_SIZE]; BOARD_SIZE],
}

impl Board {
    pub fn empty() -> Self {
        Self {
            squares: from_fn(|_| from_fn(|_| None)),
        }
    }

    pub fn standard() -> Self {
        let mut board = Self::empty();
        let mut next_id = 1;

        let back_rank = [
            PieceKind::Rook,
            PieceKind::Knight,
            PieceKind::Bishop,
            PieceKind::Queen,
            PieceKind::King,
            PieceKind::Bishop,
            PieceKind::Knight,
            PieceKind::Rook,
        ];

        for (file, kind) in back_rank.iter().copied().enumerate() {
            board.squares[0][file] = Some(Piece::new(next_id, Color::White, kind));
            next_id += 1;
            board.squares[1][file] = Some(Piece::new(next_id, Color::White, PieceKind::Pawn));
            next_id += 1;
            board.squares[6][file] = Some(Piece::new(next_id, Color::Black, PieceKind::Pawn));
            next_id += 1;
            board.squares[7][file] = Some(Piece::new(next_id, Color::Black, kind));
            next_id += 1;
        }

        board
    }

    pub fn piece_at(&self, square: Square) -> Option<&Piece> {
        self.squares[square.rank() as usize][square.file() as usize].as_ref()
    }

    pub fn piece_at_mut(&mut self, square: Square) -> Option<&mut Piece> {
        self.squares[square.rank() as usize][square.file() as usize].as_mut()
    }

    pub fn take_piece(&mut self, square: Square) -> Option<Piece> {
        self.squares[square.rank() as usize][square.file() as usize].take()
    }

    pub fn set_piece(&mut self, square: Square, piece: Option<Piece>) {
        self.squares[square.rank() as usize][square.file() as usize] = piece;
    }

    pub fn move_piece(&mut self, from: Square, to: Square) -> Result<Option<Piece>, BoardError> {
        let mut moving_piece = self
            .take_piece(from)
            .ok_or(BoardError::NoPieceAtSource(from))?;
        moving_piece.has_moved = true;

        let captured_piece = self.take_piece(to);
        if captured_piece.is_some() {
            moving_piece.capture_count += 1;
        }

        self.set_piece(to, Some(moving_piece));

        Ok(captured_piece)
    }

    pub fn print(&self) {
        println!("{self}");
    }
}

impl fmt::Display for Board {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..BOARD_SIZE).rev() {
            write!(formatter, "{} ", rank + 1)?;

            for file in 0..BOARD_SIZE {
                if file > 0 {
                    write!(formatter, " ")?;
                }

                match &self.squares[rank][file] {
                    Some(piece) => write!(formatter, "{}", piece.symbol())?,
                    None => write!(formatter, ".")?,
                }
            }

            writeln!(formatter)?;
        }

        write!(formatter, "  a b c d e f g h")
    }
}

#[cfg(test)]
mod tests {
    use super::{Board, BoardError};
    use crate::piece::{Color, Piece, PieceKind};
    use crate::square::Square;

    fn square(value: &str) -> Square {
        Square::from_algebraic(value).unwrap()
    }

    #[test]
    fn reads_piece_from_square() {
        let board = Board::standard();
        let piece = board.piece_at(square("e2")).unwrap();

        assert_eq!(piece.color, Color::White);
        assert_eq!(piece.kind, PieceKind::Pawn);
    }

    #[test]
    fn moves_piece_between_squares() {
        let mut board = Board::standard();

        let captured = board.move_piece(square("e2"), square("e4")).unwrap();

        assert_eq!(captured, None);
        assert!(board.piece_at(square("e2")).is_none());
        let moved_piece = board.piece_at(square("e4")).unwrap();
        assert_eq!(moved_piece.color, Color::White);
        assert_eq!(moved_piece.kind, PieceKind::Pawn);
        assert!(moved_piece.has_moved);
    }

    #[test]
    fn returns_captured_piece() {
        let mut board = Board::empty();

        board.set_piece(
            square("e4"),
            Some(Piece::new(1, Color::White, PieceKind::Pawn)),
        );
        board.set_piece(
            square("d5"),
            Some(Piece::new(2, Color::Black, PieceKind::Pawn)),
        );

        let captured = board.move_piece(square("e4"), square("d5")).unwrap();

        assert_eq!(captured, Some(Piece::new(2, Color::Black, PieceKind::Pawn)));

        let moved_piece = board.piece_at(square("d5")).unwrap();
        assert_eq!(moved_piece.id, 1);
        assert_eq!(moved_piece.kind, PieceKind::Pawn);
        assert_eq!(moved_piece.capture_count, 1);
    }

    #[test]
    fn rejects_moving_from_empty_square() {
        let mut board = Board::standard();
        let from = square("e4");

        let result = board.move_piece(from, square("e5"));

        assert_eq!(result, Err(BoardError::NoPieceAtSource(from)));
    }

    #[test]
    fn start_position_assigns_unique_piece_ids() {
        let board = Board::standard();
        let mut ids = Vec::new();

        for rank in 0..8 {
            for file in 0..8 {
                let square = Square::new(file, rank).unwrap();
                if let Some(piece) = board.piece_at(square) {
                    ids.push(piece.id);
                }
            }
        }

        ids.sort_unstable();
        ids.dedup();

        assert_eq!(ids.len(), 32);
    }

    #[test]
    fn moved_piece_keeps_same_id() {
        let mut board = Board::standard();
        let piece_id = board.piece_at(square("e2")).unwrap().id;

        board.move_piece(square("e2"), square("e4")).unwrap();

        assert_eq!(board.piece_at(square("e4")).unwrap().id, piece_id);
    }

    #[test]
    fn take_piece_returns_piece_with_original_id_and_kind() {
        let mut board = Board::empty();
        board.set_piece(
            square("c3"),
            Some(Piece::new(42, Color::Black, PieceKind::Grasshopper)),
        );

        let piece = board.take_piece(square("c3")).unwrap();

        assert_eq!(piece.id, 42);
        assert_eq!(piece.kind, PieceKind::Grasshopper);
        assert!(board.piece_at(square("c3")).is_none());
    }
}
