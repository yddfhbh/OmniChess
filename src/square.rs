use std::fmt;

use crate::constants::BOARD_SIZE;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Square {
    file: u8,
    rank: u8,
}

impl Square {
    pub const fn new(file: u8, rank: u8) -> Option<Self> {
        if file < BOARD_SIZE as u8 && rank < BOARD_SIZE as u8 {
            Some(Self { file, rank })
        } else {
            None
        }
    }

    pub fn from_algebraic(value: &str) -> Option<Self> {
        let bytes = value.as_bytes();

        if bytes.len() != 2 {
            return None;
        }

        let file = match bytes[0].to_ascii_lowercase() {
            b'a'..=b'h' => bytes[0].to_ascii_lowercase() - b'a',
            _ => return None,
        };

        let rank = match bytes[1] {
            b'1'..=b'8' => bytes[1] - b'1',
            _ => return None,
        };

        Self::new(file, rank)
    }

    pub const fn file(self) -> u8 {
        self.file
    }

    pub const fn rank(self) -> u8 {
        self.rank
    }
}

impl fmt::Display for Square {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file = char::from(b'a' + self.file);
        let rank = char::from(b'1' + self.rank);

        write!(formatter, "{file}{rank}")
    }
}

#[cfg(test)]
mod tests {
    use super::Square;

    #[test]
    fn parses_valid_square() {
        let square = Square::from_algebraic("e2").unwrap();

        assert_eq!(square.file(), 4);
        assert_eq!(square.rank(), 1);
        assert_eq!(square.to_string(), "e2");
    }

    #[test]
    fn accepts_uppercase_file() {
        let square = Square::from_algebraic("G8").unwrap();

        assert_eq!(square.file(), 6);
        assert_eq!(square.rank(), 7);
        assert_eq!(square.to_string(), "g8");
    }

    #[test]
    fn rejects_invalid_square() {
        assert_eq!(Square::from_algebraic("i2"), None);
        assert_eq!(Square::from_algebraic("a9"), None);
        assert_eq!(Square::from_algebraic("e22"), None);
        assert_eq!(Square::from_algebraic(""), None);
    }
}
