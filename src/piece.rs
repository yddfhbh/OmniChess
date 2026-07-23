#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub const fn opposite(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,

    // 증강 기물
    Grasshopper,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind,
}

impl Piece {
    pub const fn new(color: Color, kind: PieceKind) -> Self {
        Self { color, kind }
    }

    pub const fn symbol(self) -> char {
        match (self.color, self.kind) {
            (Color::White, PieceKind::Pawn) => 'P',
            (Color::White, PieceKind::Knight) => 'N',
            (Color::White, PieceKind::Bishop) => 'B',
            (Color::White, PieceKind::Rook) => 'R',
            (Color::White, PieceKind::Queen) => 'Q',
            (Color::White, PieceKind::King) => 'K',
            (Color::White, PieceKind::Grasshopper) => 'G',

            (Color::Black, PieceKind::Pawn) => 'p',
            (Color::Black, PieceKind::Knight) => 'n',
            (Color::Black, PieceKind::Bishop) => 'b',
            (Color::Black, PieceKind::Rook) => 'r',
            (Color::Black, PieceKind::Queen) => 'q',
            (Color::Black, PieceKind::King) => 'k',
            (Color::Black, PieceKind::Grasshopper) => 'g',
        }
    }
}
