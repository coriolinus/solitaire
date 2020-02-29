use crate::card::Card;
use std::convert::TryFrom;
use std::fmt;
use std::iter::FromIterator;

#[cfg(not(feature = "small-deck-tests"))]
pub const DECK_SIZE: usize = 54;

#[cfg(feature = "small-deck-tests")]
pub const DECK_SIZE: usize = 8;

fn is_joker(v: u8) -> bool {
    let v = v as usize;
    v == DECK_SIZE || v == (DECK_SIZE - 1)
}

#[derive(Clone)]
pub struct Deck([u8; DECK_SIZE]);

impl fmt::Debug for Deck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (idx, &card) in self.0.iter().enumerate() {
            let space = if idx == 0 { "" } else { " " };
            write!(f, "{}{}", space, card)?;
        }
        write!(f, "]")
    }
}

impl fmt::Display for Deck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (idx, &card) in self.0.iter().enumerate() {
            let space = if idx == 0 { "" } else { " " };
            write!(
                f,
                "{}{}",
                space,
                Card::try_from(card).expect("internal cards must be valid")
            )?;
        }
        write!(f, "]")
    }
}

impl Deck {
    /// Generate a new deck in sorted order
    pub fn new() -> Deck {
        let range = (1..=(DECK_SIZE as u8)).collect::<Vec<_>>();
        let mut d = [0; DECK_SIZE];
        d.copy_from_slice(&range);
        Deck(d)
    }

    // /// Generate a deck from a passphrase to create the initial deck ordering.
    // pub fn from_passphrase(phrase: &str) -> Deck {
    //     let mut deck = Deck::new();
    //     for ch in convert_text(phrase) {
    //         deck.push(Card::joker_a(), 1);
    //         deck.push(Card::joker_b(), 2);
    //         deck.triple_cut(Card::joker_a(), Card::joker_b());
    //         deck.count_cut(None);
    //         deck.count_cut(Some(ch));
    //     }
    //     deck
    // }

    pub fn shuffle(&mut self) {
        use rand::seq::SliceRandom;

        let mut rng = rand::thread_rng();
        self.0.shuffle(&mut rng);
    }

    fn find<T>(&self, card: T) -> usize
    where
        T: Into<u8>,
    {
        let needle = card.into();
        for (idx, c) in self.0.iter().enumerate() {
            if *c == needle {
                return idx;
            }
        }
        unreachable!("card not found in deck");
    }

    /// push the given card down by n spaces
    pub fn push<T>(&mut self, card: T, n: usize)
    where
        T: Into<u8>,
    {
        let n = n % DECK_SIZE;
        let idx = self.find(card);
        let mut next = self.0.clone();
        if idx + n >= DECK_SIZE {
            // wrap
            let dest_idx = (idx + n) % (DECK_SIZE - 1);
            next[dest_idx] = self.0[idx];
            next[dest_idx + 1..=idx].copy_from_slice(&self.0[dest_idx..idx]);
        } else {
            // no wrap
            next[idx..idx + n].copy_from_slice(&self.0[idx + 1..idx + n + 1]);
            next[idx + n] = self.0[idx];
        }
        self.0 = next;
    }

    /// swap the cards before the first and second found
    ///
    /// the cards specified must not be identical. The order in which the cards
    /// are found is irrelevant.
    pub fn triple_cut<T1, T2>(&mut self, card0: T1, card1: T2)
    where
        T1: Into<u8>,
        T2: Into<u8>,
    {
        let card0 = card0.into();
        let card1 = card1.into();
        assert_ne!(card0, card1, "cards mut not be identical");
        let (idx0, idx1) = {
            let mut idx0 = self.find(card0);
            let mut idx1 = self.find(card1);
            if idx0 > idx1 {
                std::mem::swap(&mut idx0, &mut idx1);
            }
            (idx0, idx1)
        };
        debug_assert!(idx0 < idx1, "triple cut indices are backwards");
        let mut next = [0; DECK_SIZE];
        let new_idx0 = DECK_SIZE - idx1 - 1;
        let new_idx1 = DECK_SIZE - idx0 - 1;
        debug_assert_eq!(
            idx1 - idx0,
            new_idx1 - new_idx0,
            "center range must have constant size"
        );
        next[..new_idx0].copy_from_slice(&self.0[idx1 + 1..]);
        next[new_idx0..=new_idx1].copy_from_slice(&self.0[idx0..=idx1]);
        next[new_idx1 + 1..].copy_from_slice(&self.0[..idx0]);
        self.0 = next;
    }

    /// excluding the bottom card of the deck, cut the deck at a position
    /// specified by the bottom card
    pub fn count_cut(&mut self, override_idx: Option<u8>) {
        let idx = match override_idx {
            Some(oi) => oi as usize,
            None => {
                let mut idx = self.0[DECK_SIZE - 1] as usize;
                // the jokers should both have the same value
                if idx == DECK_SIZE {
                    idx -= 1;
                }
                idx
            }
        };

        let range_b_len = DECK_SIZE - idx - 1;
        let mut next = [0; DECK_SIZE];
        next[..range_b_len].copy_from_slice(&self.0[idx..DECK_SIZE - 1]);
        next[range_b_len..DECK_SIZE - 1].copy_from_slice(&self.0[..idx]);
        next[DECK_SIZE - 1] = self.0[DECK_SIZE - 1];
        self.0 = next;
    }

    /// find the output card's value given the current deck state
    ///
    /// range: `1..=DECK_SIZE`
    pub fn output(&self) -> Option<u8> {
        let idx = {
            let mut idx = self.0[0] as usize;
            // the jokers should both have the same value
            if idx == DECK_SIZE {
                idx -= 1;
            }
            idx
        };
        let card = self.0[idx];
        if is_joker(card) {
            None
        } else {
            Some(card)
        }
    }
}

impl Default for Deck {
    fn default() -> Deck {
        Deck::new()
    }
}

impl<T> FromIterator<T> for Deck
where
    T: Into<u8>,
{
    fn from_iter<I>(iter: I) -> Deck
    where
        I: IntoIterator<Item = T>,
    {
        let cards: Vec<u8> = iter
            .into_iter()
            .take(DECK_SIZE + 1)
            .map(Into::into)
            .collect();
        assert_eq!(cards.len(), DECK_SIZE, "input had incorrect length");
        debug_assert_eq!(
            {
                let mut scards = cards.clone();
                scards.sort();
                scards.dedup();
                scards.len()
            },
            DECK_SIZE,
            "input had duplicate items"
        );
        let mut arr = [0; DECK_SIZE];
        arr.copy_from_slice(&cards);
        Deck(arr)
    }
}

#[cfg(all(test, feature = "small-deck-tests"))]
mod small_deck_tests {
    use super::*;

    #[test]
    fn test_push_1_no_overflow() {
        let mut deck = Deck::new();
        assert_eq!(deck.0, [1, 2, 3, 4, 5, 6, 7, 8]);
        deck.push(2, 1);
        assert_eq!(deck.0, [1, 3, 2, 4, 5, 6, 7, 8]);
        deck.push(2, 1);
        assert_eq!(deck.0, [1, 3, 4, 2, 5, 6, 7, 8]);
    }

    #[test]
    fn test_push_1_overflow() {
        let mut deck = Deck::new();
        assert_eq!(deck.0, [1, 2, 3, 4, 5, 6, 7, 8]);
        deck.push(7, 1);
        assert_eq!(deck.0, [1, 2, 3, 4, 5, 6, 8, 7]);
        deck.push(7, 1);
        assert_eq!(deck.0, [1, 7, 2, 3, 4, 5, 6, 8]);
        deck.push(6, 1);
        deck.push(6, 1);
        assert_eq!(deck.0, [1, 6, 7, 2, 3, 4, 5, 8]);
    }

    #[test]
    fn test_push_2_no_overflow() {
        let mut deck = Deck::new();
        assert_eq!(deck.0, [1, 2, 3, 4, 5, 6, 7, 8]);
        deck.push(2, 2);
        assert_eq!(deck.0, [1, 3, 4, 2, 5, 6, 7, 8]);
        deck.push(4, 2);
        assert_eq!(deck.0, [1, 3, 2, 5, 4, 6, 7, 8]);
    }

    #[test]
    fn test_push_2_overflow() {
        let mut deck = Deck::new();
        assert_eq!(deck.0, [1, 2, 3, 4, 5, 6, 7, 8]);
        deck.push(8, 2);
        assert_eq!(deck.0, [1, 2, 8, 3, 4, 5, 6, 7]);
        deck.push(6, 2);
        assert_eq!(deck.0, [1, 6, 2, 8, 3, 4, 5, 7]);
    }

    #[test]
    fn test_triple_cut() {
        let mut deck = Deck::new();
        assert_eq!(deck.0, [1, 2, 3, 4, 5, 6, 7, 8]);
        // basic swap
        deck.triple_cut(3, 6);
        assert_eq!(deck.0, [7, 8, 3, 4, 5, 6, 1, 2]);
        // unbalanced + end left
        deck.triple_cut(7, 1);
        assert_eq!(deck.0, [2, 7, 8, 3, 4, 5, 6, 1]);
        // unbalanced + end right + ordering
        deck.triple_cut(1, 8);
        assert_eq!(deck.0, [8, 3, 4, 5, 6, 1, 2, 7]);
    }

    #[test]
    fn test_count_cut() {
        let mut deck: Deck = (1..=(DECK_SIZE as u8)).rev().collect();
        assert_eq!(deck.0, [8, 7, 6, 5, 4, 3, 2, 1]);
        deck.count_cut(None);
        assert_eq!(deck.0, [7, 6, 5, 4, 3, 2, 8, 1]);
        deck.push(2, 2);
        assert_eq!(deck.0, [7, 6, 5, 4, 3, 8, 1, 2]);
        deck.count_cut(None);
        assert_eq!(deck.0, [5, 4, 3, 8, 1, 7, 6, 2]);
        deck.push(3, 5);
        assert_eq!(deck.0, [5, 4, 8, 1, 7, 6, 2, 3]);
        deck.count_cut(None);
        assert_eq!(deck.0, [1, 7, 6, 2, 5, 4, 8, 3]);
    }

    #[test]
    fn test_count_cut_joker_semantics() {
        let mut deck = Deck::new();
        assert_eq!(deck.0, [1, 2, 3, 4, 5, 6, 7, 8]);
        deck.count_cut(None);
        assert_eq!(deck.0, [1, 2, 3, 4, 5, 6, 7, 8]);
    }
}

#[cfg(all(test, not(feature = "small-deck-tests")))]
mod tests {
    use super::*;

    #[test]
    fn test_unkeyed() {
        let d = Deck::new();
        println!("{:?}", d);
        assert!(is_joker(d.0[DECK_SIZE - 1]));
        assert!(is_joker(d.0[DECK_SIZE - 2]));
    }
}
