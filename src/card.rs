use crate::deck::DECK_SIZE;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

const SUIT_SIZE: u8 = 13;

pub const JOKER_A: Card = Card {
    suit: Suit::Joker,
    rank: Rank::Number(1),
};

pub const JOKER_B: Card = Card {
    suit: Suit::Joker,
    rank: Rank::Number(2),
};

#[derive(Error, Debug)]
pub enum CardConversionError {
    #[error("value out of range")]
    ValueOutOfRange,
    #[error("unknown suit")]
    UnknownSuit,
    #[error("wrong length: need [2..3]; got {0}")]
    WrongLength(usize),
    #[error("failed to parse card portion as utf8")]
    LastByteUtf8(#[from] std::str::Utf8Error),
    #[error("could not parse rank")]
    CouldNotParseRank(#[from] std::num::ParseIntError),
    #[error("unknown joker: need A or B; got {0}")]
    UnknownJoker(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Suit {
    Club,
    Diamond,
    Heart,
    Spade,
    Joker,
}

impl TryFrom<u8> for Suit {
    type Error = CardConversionError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == 0 || value as usize > DECK_SIZE {
            return Err(CardConversionError::ValueOutOfRange);
        }
        use Suit::*;
        Ok(match (value - 1) / SUIT_SIZE {
            0 => Club,
            1 => Diamond,
            2 => Heart,
            3 => Spade,
            4 => Joker,
            _ => unreachable!("already filtered out high values"),
        })
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Suit::*;
        write!(
            f,
            "{}",
            match self {
                Club => "♧",
                Diamond => "♢",
                Heart => "♡",
                Spade => "♤",
                Joker => "J",
            }
        )
    }
}

impl Suit {
    pub fn to_ascii_string(&self) -> String {
        use Suit::*;
        format!(
            "{}",
            match self {
                Club => "C",
                Diamond => "D",
                Heart => "H",
                Spade => "S",
                Joker => "J",
            }
        )
    }
}

impl FromStr for Suit {
    type Err = CardConversionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Suit::*;
        match s {
            "C" | "c" | "♣" | "♧" => Ok(Club),
            "D" | "d" | "♦" | "♢" => Ok(Diamond),
            "H" | "h" | "♥" | "♡" => Ok(Heart),
            "S" | "s" | "♠" | "♤" => Ok(Spade),
            "J" | "j" => Ok(Joker),
            _ => Err(CardConversionError::UnknownSuit),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rank {
    Number(u8),
    Jack,
    Queen,
    King,
}

impl Rank {
    pub fn value(&self) -> u8 {
        use Rank::*;
        match self {
            Number(n) => *n,
            Jack => 11,
            Queen => 12,
            King => 13,
        }
    }
}

impl TryFrom<u8> for Rank {
    type Error = CardConversionError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == 0 || value as usize > crate::deck::DECK_SIZE {
            return Err(CardConversionError::ValueOutOfRange);
        }
        use Rank::*;
        Ok(match ((value - 1) % SUIT_SIZE) + 1 {
            n @ 1..=10 => Number(n),
            11 => Jack,
            12 => Queen,
            13 => King,
            _ => unreachable!("invalid rank number"),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Card {
    suit: Suit,
    rank: Rank,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.rank_string(), self.suit)
    }
}

impl Card {
    fn rank_string(&self) -> String {
        use Rank::*;
        match self {
            &JOKER_A => "A".to_string(),
            &JOKER_B => "B".to_string(),
            _ => {
                let mut s = String::with_capacity(2);
                match self.rank {
                    Number(n) => s.push_str(&n.to_string()),
                    Jack => s.push('J'),
                    Queen => s.push('Q'),
                    King => s.push('K'),
                }
                s
            }
        }
    }
    pub fn to_ascii_string(&self) -> String {
        format!("{}{}", self.rank_string(), self.suit.to_ascii_string())
    }
}

impl FromStr for Card {
    type Err = CardConversionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let suit_s = &s
            .chars()
            .last()
            .ok_or(CardConversionError::WrongLength(0))?
            .to_string();
        let suit: Suit = str::parse(&suit_s)?;

        let rank_s = &s[..s.len() - suit_s.len()];
        match (suit, rank_s) {
            (Suit::Joker, "A") | (Suit::Joker, "a") => Ok(JOKER_A),
            (Suit::Joker, "B") | (Suit::Joker, "b") => Ok(JOKER_B),
            (Suit::Joker, _) => Err(CardConversionError::UnknownJoker(rank_s.into())),
            _ => {
                use Rank::*;
                let rank = match rank_s {
                    "11" | "J" | "j" => Jack,
                    "12" | "Q" | "q" => Queen,
                    "13" | "K" | "k" => King,
                    "A" | "a" => Number(1),
                    _ => Number(str::parse(rank_s)?),
                };
                Ok(Card::new(suit, rank))
            }
        }
    }
}

impl Default for Card {
    fn default() -> Self {
        Card {
            suit: Suit::Club,
            rank: Rank::Number(1),
        }
    }
}

impl Card {
    pub fn new(suit: Suit, rank: Rank) -> Card {
        Card { suit, rank }
    }

    pub fn suit(&self) -> Suit {
        self.suit
    }

    pub fn rank(&self) -> Rank {
        self.rank
    }
}

impl From<Card> for u8 {
    fn from(card: Card) -> Self {
        let suit_value = card.suit as u8 * SUIT_SIZE;
        suit_value + card.rank.value()
    }
}

impl From<&Card> for u8 {
    fn from(card: &Card) -> Self {
        u8::from(*card)
    }
}

impl TryFrom<u8> for Card {
    type Error = CardConversionError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let suit = Suit::try_from(value)?;
        let rank = Rank::try_from(value)?;
        Ok(Card::new(suit, rank))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reversing() {
        for i in 1..=(DECK_SIZE as u8) {
            let card = Card::try_from(i).unwrap();
            let v = u8::from(card);
            dbg!(i, card, v);
            assert_eq!(i, v, "reversing from u8 must match")
        }
    }

    #[test]
    fn test_invalid_cards() {
        assert!(Card::try_from(0).is_err());
        assert!(Card::try_from(DECK_SIZE as u8 + 1).is_err());
    }

    #[test]
    #[cfg(not(feature = "small-deck-tests"))]
    fn test_jokers() {
        for &joker in &[JOKER_A, JOKER_B] {
            let u = u8::from(joker);
            assert!(u > 0);
            assert!(dbg!(u) <= dbg!(DECK_SIZE) as u8);
            let c = Card::try_from(u).unwrap();
            assert_eq!(c, joker);
        }
    }

    #[test]
    fn test_parse() {
        for i in 1..=(DECK_SIZE as u8) {
            let card = Card::try_from(i).unwrap();
            let s = card.to_string();
            dbg!(card, &s);
            let parsed = Card::from_str(&s).unwrap();
            assert_eq!(card, parsed);
        }
    }
}
