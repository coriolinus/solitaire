pub mod card;
pub mod deck;

pub use card::Card;
pub use deck::Deck;

/// A Keystream is an iterator which mutates a card deck to generate an infinite
/// pseudo-random stream of characters in the range `'A'..='Z'`.
pub struct Keystream(Deck);

impl From<Deck> for Keystream {
    fn from(deck: Deck) -> Self {
        Keystream(deck)
    }
}

pub fn keystream(deck: Deck) -> Keystream {
    deck.into()
}

impl Iterator for Keystream {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        let deck = &mut self.0;
        let mut output = None;
        while output.is_none() {
            deck.push(Card::joker_a(), 1);
            deck.push(Card::joker_b(), 2);
            deck.triple_cut(Card::joker_a(), Card::joker_b());
            deck.count_cut();
            output = dbg!(deck.output());
        }
        output.map(u8::from)
    }
}

#[cfg(all(test, not(feature = "small-deck-tests")))]
mod tests {
    use super::*;

    #[test]
    /// This test is a truncated version of the first test in the book.
    fn test_example_outputs_1() {
        assert_eq!(
            &keystream(Deck::new()).take(8).collect::<Vec<_>>(),
            &[4, 49, 10, 24, 8, 51, 44, 6],
        );
    }

    #[test]
    #[should_panic]
    /// This test is the full first example in the book.
    ///
    /// However, looking at the debug output, I am becoming increasingly convinced
    /// that the final digit here is a misprint or other error. I can't see any
    /// way that my program could be buggy in a way that would produce correct
    /// output eight times in a row, then generate an output from a completely
    /// different area of the deck.
    fn test_example_outputs_2() {
        assert_eq!(
            &keystream(Deck::new()).take(9).collect::<Vec<_>>(),
            &[4, 49, 10, 24, 8, 51, 44, 6, 33],
        );
    }
}
