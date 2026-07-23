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
    Grasshopper,
}

pub type PieceId = u32;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PieceFlags {
    pub dodge_available: bool,
    pub protected: bool,
    pub frozen: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    pub id: PieceId,
    pub color: Color,
    pub kind: PieceKind,
    pub has_moved: bool,
    pub capture_count: u8,
    pub hp: Option<u8>,
    pub flags: PieceFlags,
}

impl Piece {
    pub fn new(id: PieceId, color: Color, kind: PieceKind) -> Self {
        Self {
            id,
            color,
            kind,
            has_moved: false,
            capture_count: 0,
            hp: None,
            flags: PieceFlags::default(),
        }
    }

    pub const fn symbol(&self) -> char {
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
