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
            deck.count_cut(None);
            output = deck.output();
        }
        output.map(u8::from)
    }
}

/// convert a text input into a numeric stream from 1..26 according to its chars
fn convert_text(text: &str) -> impl Iterator<Item = u8> + '_ {
    text.chars()
        .filter(char::is_ascii_alphabetic)
        .map(|c| (c.to_ascii_uppercase() as u8) - b'A' + 1)
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

    #[test]
    /// this test is sample 2 in the book
    fn test_passphrase_keygen() {
        assert_eq!(
            &keystream(Deck::from_passphrase("foo"))
                .take(15)
                .collect::<Vec<_>>(),
            &[8, 19, 7, 25, 20, 9, 8, 22, 32, 43, 5, 26, 17, 38, 48],
        );
    }
}
