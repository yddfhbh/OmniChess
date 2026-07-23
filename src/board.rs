use crate::piece::{Color, Piece, PieceKind};
use crate::square::Square;

pub const BOARD_SIZE: usize = 8;

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
            squares: [[None; BOARD_SIZE]; BOARD_SIZE],
        }
    }

    pub fn standard() -> Self {
        let mut board = Self::empty();

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

        for file in 0..BOARD_SIZE {
            board.squares[0][file] = Some(Piece::new(Color::White, back_rank[file]));
            board.squares[1][file] = Some(Piece::new(Color::White, PieceKind::Pawn));

            board.squares[6][file] = Some(Piece::new(Color::Black, PieceKind::Pawn));
            board.squares[7][file] = Some(Piece::new(Color::Black, back_rank[file]));
        }

        board
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        self.squares[square.rank() as usize][square.file() as usize]
    }

    pub fn set_piece(&mut self, square: Square, piece: Option<Piece>) {
        self.squares[square.rank() as usize][square.file() as usize] = piece;
    }

    pub fn move_piece(&mut self, from: Square, to: Square) -> Result<Option<Piece>, BoardError> {
        let moving_piece = self
            .piece_at(from)
            .ok_or(BoardError::NoPieceAtSource(from))?;

        let captured_piece = self.piece_at(to);

        self.set_piece(from, None);
        self.set_piece(to, Some(moving_piece));

        Ok(captured_piece)
    }

    pub fn print(&self) {
        println!();

        for rank in (0..BOARD_SIZE).rev() {
            print!("{} ", rank + 1);

            for file in 0..BOARD_SIZE {
                match self.squares[rank][file] {
                    Some(piece) => print!("{} ", piece.symbol()),
                    None => print!(". "),
                }
            }

            println!();
        }

        println!("  a b c d e f g h");
        println!();
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

        assert_eq!(piece, Piece::new(Color::White, PieceKind::Pawn));
    }

    #[test]
    fn moves_piece_between_squares() {
        let mut board = Board::standard();

        let captured = board.move_piece(square("e2"), square("e4")).unwrap();

        assert_eq!(captured, None);
        assert_eq!(board.piece_at(square("e2")), None);
        assert_eq!(
            board.piece_at(square("e4")),
            Some(Piece::new(Color::White, PieceKind::Pawn))
        );
    }

    #[test]
    fn returns_captured_piece() {
        let mut board = Board::empty();

        board.set_piece(
            square("e4"),
            Some(Piece::new(Color::White, PieceKind::Pawn)),
        );

        board.set_piece(
            square("d5"),
            Some(Piece::new(Color::Black, PieceKind::Pawn)),
        );

        let captured = board.move_piece(square("e4"), square("d5")).unwrap();

        assert_eq!(captured, Some(Piece::new(Color::Black, PieceKind::Pawn)));

        assert_eq!(
            board.piece_at(square("d5")),
            Some(Piece::new(Color::White, PieceKind::Pawn))
        );
    }

    #[test]
    fn rejects_moving_from_empty_square() {
        let mut board = Board::standard();
        let from = square("e4");

        let result = board.move_piece(from, square("e5"));

        assert_eq!(result, Err(BoardError::NoPieceAtSource(from)));
    }
}
