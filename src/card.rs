use crate::deck::DECK_SIZE;
use std::convert::TryFrom;
use std::fmt;

const SUIT_SIZE: u8 = 13;

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
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == 0 || value as usize > DECK_SIZE {
            return Err("value out of range");
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
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == 0 || value as usize > crate::deck::DECK_SIZE {
            return Err("value out of range");
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
        use Rank::*;
        let rank = if self.suit == Suit::Joker {
            match self.rank {
                Number(0) => String::from("A"),
                Number(1) => String::from("B"),
                _ => panic!("invalid joker"),
            }
        } else {
            let mut s = String::with_capacity(2);
            match self.rank {
                Number(n) => s.push_str(&n.to_string()),
                Jack => s.push('J'),
                Queen => s.push('Q'),
                King => s.push('K'),
            }
            s
        };

        write!(f, "{}{}", rank, self.suit)
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

    pub const fn joker_a() -> Card {
        Card {
            suit: Suit::Joker,
            rank: Rank::Number(1),
        }
    }

    pub const fn joker_b() -> Card {
        Card {
            suit: Suit::Joker,
            rank: Rank::Number(2),
        }
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

impl TryFrom<u8> for Card {
    type Error = &'static str;
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
        for &joker in &[Card::joker_a(), Card::joker_b()] {
            let u = u8::from(joker);
            assert!(u > 0);
            assert!(dbg!(u) <= dbg!(DECK_SIZE) as u8);
            let c = Card::try_from(u).unwrap();
            assert_eq!(c, joker);
        }
    }
}
