use std::fmt;

use crate::piece::PieceKind;
use crate::square::Square;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoveAction {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<PieceKind>,
}

impl MoveAction {
    pub fn from_uci(value: &str) -> Option<Self> {
        if !value.is_ascii() || !(value.len() == 4 || value.len() == 5) {
            return None;
        }

        let from = Square::from_algebraic(&value[0..2])?;
        let to = Square::from_algebraic(&value[2..4])?;

        let promotion = if value.len() == 5 {
            match value.as_bytes()[4].to_ascii_lowercase() {
                b'q' => Some(PieceKind::Queen),
                b'r' => Some(PieceKind::Rook),
                b'b' => Some(PieceKind::Bishop),
                b'n' => Some(PieceKind::Knight),
                _ => return None,
            }
        } else {
            None
        };

        Some(Self {
            from,
            to,
            promotion,
        })
    }
}

impl fmt::Display for MoveAction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{}", self.from, self.to)?;

        if let Some(kind) = self.promotion {
            let symbol = match kind {
                PieceKind::Queen => 'q',
                PieceKind::Rook => 'r',
                PieceKind::Bishop => 'b',
                PieceKind::Knight => 'n',
                _ => return Err(fmt::Error),
            };

            write!(formatter, "{symbol}")?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Move(MoveAction),
}

impl Action {
    pub fn from_uci(value: &str) -> Option<Self> {
        Some(Self::Move(MoveAction::from_uci(value)?))
    }
}

impl fmt::Display for Action {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Move(action) => write!(formatter, "{action}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, MoveAction};
    use crate::piece::PieceKind;
    use crate::square::Square;

    fn square(value: &str) -> Square {
        Square::from_algebraic(value).unwrap()
    }

    #[test]
    fn parses_normal_move() {
        let action = MoveAction::from_uci("e2e4").unwrap();

        assert_eq!(action.from, square("e2"));
        assert_eq!(action.to, square("e4"));
        assert_eq!(action.promotion, None);
        assert_eq!(action.to_string(), "e2e4");
    }

    #[test]
    fn parses_promotion_move() {
        let action = MoveAction::from_uci("a7a8q").unwrap();

        assert_eq!(action.promotion, Some(PieceKind::Queen));
        assert_eq!(action.to_string(), "a7a8q");
    }

    #[test]
    fn rejects_invalid_move_text() {
        assert_eq!(Action::from_uci("e2"), None);
        assert_eq!(Action::from_uci("e2e9"), None);
        assert_eq!(Action::from_uci("hello"), None);
        assert_eq!(Action::from_uci("a7a8k"), None);
    }
}
