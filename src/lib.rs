pub mod card;
pub mod deck;

pub use card::Card;
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
    type Item = char;
    fn next(&mut self) -> Option<char> {
        let deck = &mut self.0;
        let mut output = None;
        while output.is_none() {
            deck.push(Card::joker_a(), 1);
            deck.push(Card::joker_b(), 2);
            deck.triple_cut(Card::joker_a(), Card::joker_b());
            deck.count_cut();
            output = deck.output();
        }
        output.map(|card| card.char().expect("deck.output() should exclude jokers"))
    }
}
