pub mod card;
pub mod deck;

pub use card::{Card, Value};
pub use deck::Deck;

/// A Keystream is an iterator which mutates a card deck to generate an infinite
/// pseudo-random stream of characters in the range `'A'..='Z'`.
pub struct Keystream(Deck<Card>);

impl From<Deck<Card>> for Keystream {
    fn from(deck: Deck<Card>) -> Self {
        Keystream(deck)
    }
}

pub fn keystream(deck: Deck<Card>) -> Keystream {
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
            output = deck.output();
        }
        output.map(|card| card.value() as u8)
    }
}

#[cfg(all(test, not(feature = "small-deck-tests")))]
mod tests {
    use super::*;

    #[test]
    fn test_example_outputs_1() {
        assert_eq!(
            &keystream(Deck::new_unshuffled())
                .take(9)
                .collect::<Vec<_>>(),
            &[4, 49, 10, 24, 8, 51, 44, 6, 33],
        );
    }
}
