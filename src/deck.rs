use crate::card::{Card, Value};
use std::fmt;

#[cfg(not(feature = "small-deck-tests"))]
pub const DECK_SIZE: usize = 54;

#[cfg(feature = "small-deck-tests")]
pub const DECK_SIZE: usize = 8;

#[derive(Clone)]
pub struct Deck<T>([T; DECK_SIZE]);

impl<T> fmt::Debug for Deck<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (idx, card) in self.0.iter().enumerate() {
            let space = if idx == 0 { "" } else { " " };
            write!(f, "{}{:?}", space, card)?;
        }
        write!(f, "]")
    }
}

impl<T> fmt::Display for Deck<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (idx, card) in self.0.iter().enumerate() {
            let space = if idx == 0 { "" } else { " " };
            write!(f, "{}{}", space, card)?;
        }
        write!(f, "]")
    }
}

impl<T> Deck<T>
where
    T: Copy + Default,
{
    pub fn new<I>(cards: I) -> Deck<T>
    where
        I: IntoIterator,
        I::Item: Into<T>,
    {
        let cards: Vec<T> = cards.into_iter().map(|card| card.into()).collect();
        assert_eq!(cards.len(), DECK_SIZE, "input had incorrect length");
        let mut arr = [T::default(); DECK_SIZE];
        arr.copy_from_slice(&cards);
        Deck(arr)
    }
}

impl Deck<Card> {
    /// Generate a new deck in sorted order
    #[cfg(not(feature = "small-deck-tests"))]
    pub fn new_unshuffled() -> Deck<Card> {
        use crate::card::Rank::*;
        use crate::card::Suit::*;

        let mut d = [Card::default(); DECK_SIZE];
        let mut idx = 0;
        for &suit in &[Club, Diamond, Heart, Spade] {
            for n in 1..=10 {
                d[idx] = Card::new(suit, Number(n));
                idx += 1;
            }
            d[idx] = Card::new(suit, Jack);
            idx += 1;
            d[idx] = Card::new(suit, Queen);
            idx += 1;
            d[idx] = Card::new(suit, King);
            idx += 1;
        }
        d[idx] = Card::joker_a();
        idx += 1;

        debug_assert!(d[..idx]
            .windows(2)
            .all(|w| w[0].value() + 1 == w[1].value()));

        d[idx] = Card::joker_b();
        debug_assert_eq!(idx, DECK_SIZE - 1, "must have filled the deck");

        Deck(d)
    }

    /// find the output card given the current deck state
    pub fn output(&self) -> Option<Card> {
        let idx = self.0[0].value();
        use crate::card::Suit::Joker;
        if self.0[idx].suit() == Joker {
            None
        } else {
            Some(self.0[idx])
        }
    }
}

impl<T> Deck<T>
where
    T: Value + Copy + Default,
{
    /// excluding the bottom card of the deck, cut the deck at a position
    /// specified by the bottom card
    pub fn count_cut(&mut self) {
        let idx = self.0[DECK_SIZE - 1].value();
        let range_b_len = DECK_SIZE - idx - 1;
        let mut next = [T::default(); DECK_SIZE];
        next[..range_b_len].copy_from_slice(&self.0[idx..DECK_SIZE - 1]);
        next[range_b_len..DECK_SIZE - 1].copy_from_slice(&self.0[..idx]);
        next[DECK_SIZE - 1] = self.0[DECK_SIZE - 1];
        self.0 = next;
    }
}

impl<T> Deck<T>
where
    T: PartialEq + Copy,
{
    fn find(&self, card: T) -> usize {
        for (idx, c) in self.0.iter().enumerate() {
            if *c == card {
                return idx;
            }
        }
        unreachable!("card not found in deck");
    }

    /// push the given card down by n spaces
    pub fn push(&mut self, card: T, n: usize) {
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
}

impl<T> Deck<T> {
    pub fn shuffle(&mut self) {
        use rand::seq::SliceRandom;

        let mut rng = rand::thread_rng();
        self.0.shuffle(&mut rng);
    }
}

impl<T> Deck<T>
where
    T: Copy + Default + PartialEq + std::fmt::Debug,
{
    /// swap the cards before the first and second found
    ///
    /// the cards specified must not be identical. The order in which the cards
    /// are found is irrelevant.
    pub fn triple_cut(&mut self, card0: T, card1: T) {
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
        let mut next = [T::default(); DECK_SIZE];
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
}

#[cfg(all(test, feature = "small-deck-tests"))]
mod small_deck_tests {
    use super::*;

    #[test]
    fn test_push_1_no_overflow() {
        let mut deck = Deck::new(1..=DECK_SIZE);
        assert_eq!(deck.0, [1, 2, 3, 4, 5, 6, 7, 8]);
        deck.push(2, 1);
        assert_eq!(deck.0, [1, 3, 2, 4, 5, 6, 7, 8]);
        deck.push(2, 1);
        assert_eq!(deck.0, [1, 3, 4, 2, 5, 6, 7, 8]);
    }

    #[test]
    fn test_push_1_overflow() {
        let mut deck = Deck::new(1..=DECK_SIZE);
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
        let mut deck = Deck::new(1..=DECK_SIZE);
        assert_eq!(deck.0, [1, 2, 3, 4, 5, 6, 7, 8]);
        deck.push(2, 2);
        assert_eq!(deck.0, [1, 3, 4, 2, 5, 6, 7, 8]);
        deck.push(4, 2);
        assert_eq!(deck.0, [1, 3, 2, 5, 4, 6, 7, 8]);
    }

    #[test]
    fn test_push_2_overflow() {
        let mut deck = Deck::new(1..=DECK_SIZE);
        assert_eq!(deck.0, [1, 2, 3, 4, 5, 6, 7, 8]);
        deck.push(8, 2);
        assert_eq!(deck.0, [1, 2, 8, 3, 4, 5, 6, 7]);
        deck.push(6, 2);
        assert_eq!(deck.0, [1, 6, 2, 8, 3, 4, 5, 7]);
    }

    #[test]
    fn test_triple_cut() {
        let mut deck = Deck::new(1..=DECK_SIZE);
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
        let mut deck = Deck::new((1..=DECK_SIZE).rev());
        assert_eq!(deck.0, [8, 7, 6, 5, 4, 3, 2, 1]);
        deck.count_cut();
        assert_eq!(deck.0, [7, 6, 5, 4, 3, 2, 8, 1]);
        deck.push(2, 2);
        assert_eq!(deck.0, [7, 6, 5, 4, 3, 8, 1, 2]);
        deck.count_cut();
        assert_eq!(deck.0, [5, 4, 3, 8, 1, 7, 6, 2]);
        deck.push(3, 5);
        assert_eq!(deck.0, [5, 4, 8, 1, 7, 6, 2, 3]);
        deck.count_cut();
        assert_eq!(deck.0, [1, 7, 6, 2, 5, 4, 8, 3]);
    }
}

#[cfg(all(test, not(feature = "small-deck-tests")))]
mod tests {
    use super::*;

    #[test]
    fn test_unkeyed() {
        let d = Deck::new_unshuffled();
        println!("{:?}", d);
        assert_eq!(d.0[DECK_SIZE - 1].suit(), crate::card::Suit::Joker);
        assert_eq!(d.0[DECK_SIZE - 2].suit(), crate::card::Suit::Joker);
    }
}
