use crate::card::Card;

pub const DECK_SIZE: usize = 54;
pub struct Deck<T>([T; DECK_SIZE]);

impl Deck<Card> {
    /// Generate a new deck in sorted order
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

    /// excluding the bottom card of the deck, cut the deck at a position
    /// specified by the bottom card
    pub fn count_cut(&mut self) {
        let idx = self.0[DECK_SIZE - 1].value();
        let range_b_len = DECK_SIZE - idx - 1;
        let mut next = [Card::default(); DECK_SIZE];
        next[..range_b_len].copy_from_slice(&self.0[idx..DECK_SIZE - 1]);
        next[range_b_len..DECK_SIZE - 1].copy_from_slice(&self.0[..idx]);
        next[DECK_SIZE - 1] = self.0[DECK_SIZE - 1];
        self.0 = next;
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
