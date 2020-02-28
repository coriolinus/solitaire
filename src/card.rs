use std::fmt;

pub trait Value {
    fn value(&self) -> usize;
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
    pub fn value(&self) -> usize {
        use Rank::*;
        match self {
            Number(n) => *n as usize,
            Jack => 11,
            Queen => 12,
            King => 13,
        }
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
            rank: Rank::Number(0),
        }
    }

    pub const fn joker_b() -> Card {
        Card {
            suit: Suit::Joker,
            rank: Rank::Number(1),
        }
    }

    pub fn suit(&self) -> Suit {
        self.suit
    }

    pub fn rank(&self) -> Rank {
        self.rank
    }
}

impl Value for Card {
    fn value(&self) -> usize {
        if self.suit == Suit::Joker {
            53
        } else {
            let suit_value = self.suit as usize * 13;
            suit_value + self.rank.value()
        }
    }
}

impl Value for usize {
    fn value(&self) -> usize {
        *self
    }
}
