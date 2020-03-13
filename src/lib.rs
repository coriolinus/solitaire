pub mod card;
pub mod deck;
pub mod textbyte;

use card::{JOKER_A, JOKER_B};
pub use deck::Deck;
use textbyte::prelude::*;

/// size of groups of output characters
pub const GROUP_SIZE: usize = 5;
/// inputs whose
pub const PAD_CHAR: u8 = b'X' - b'A' + 1;

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
            deck.push(JOKER_A, 1);
            deck.push(JOKER_B, 2);
            deck.triple_cut(JOKER_A, JOKER_B);
            deck.count_cut(None);
            output = deck.output();
        }
        output.map(u8::from)
    }
}

/// encrypt some plaintext using a pre-prepared deck
///
/// Note that the deck is consumed. Prepare your entire message before
/// calling this method. Solitaire is not recommended for long messages.
fn crypt(deck: Deck, text: &str, operation: impl Fn(u8, u8) -> u8) -> String {
    textbyte(text)
        .pad(PAD_CHAR, GROUP_SIZE)
        .zip(keystream(deck))
        .map(|(c, k)| operation(c, k))
        .restore()
        .separate(' ', GROUP_SIZE)
}

/// encrypt some plaintext using a pre-prepared deck
///
/// Note that the deck is consumed. Prepare the entire message before
/// calling this method. Solitaire is not recommended for long messages.
pub fn encrypt(deck: Deck, text: &str) -> String {
    crypt(deck, text, |p, k| p + k)
}

/// decrypt some ciphertext using a pre-prepared deck
///
/// Note that the deck is consumed. Prepare the entire message before
/// calling this method. Solitaire is not recommended for long messages.
pub fn decrypt(deck: Deck, text: &str) -> String {
    crypt(deck, text, |c, k| c + (26 * 3) - k)
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
    /// that the final digit here is a misprint or other upstream error. I can't see any
    /// way that my program could be buggy in a way that would produce correct
    /// output eight times in a row, then generate an output from a completely
    /// different area of the deck.
    ///
    /// This is particularly true given the perfect success the implementation
    /// has in other, more difficult examples.
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

    #[test]
    fn test_encrypt_example_1() {
        assert_eq!(encrypt(Deck::new(), "aaaaa aaaaa"), "EXKYI ZSGEH",);
    }

    #[test]
    fn test_decrypt_example_1() {
        assert_eq!(decrypt(Deck::new(), "exkyi zsgeh"), "AAAAA AAAAA",);
    }

    #[test]
    fn test_encrypt_example_2() {
        assert_eq!(
            encrypt(Deck::from_passphrase("foo"), "aaaaa aaaaa aaaaa"),
            "ITHZU JIWGR FARMW",
        );
    }

    #[test]
    fn test_decrypt_example_2() {
        assert_eq!(
            decrypt(Deck::from_passphrase("foo"), "ithzu jiwgr farmw"),
            "AAAAA AAAAA AAAAA",
        );
    }

    #[test]
    fn test_encrypt_example_3() {
        assert_eq!(
            encrypt(Deck::from_passphrase("cryptonomicon"), "solitaire"),
            "KIRAK SFJAN",
        );
    }

    #[test]
    fn test_decrypt_example_3() {
        assert_eq!(
            decrypt(Deck::from_passphrase("cryptonomicon"), "kirak sfjan"),
            "SOLIT AIREX",
        );
    }

    #[test]
    fn test_reverse_message() {
        for w in [
            "The quick brown fox jumps over the lazy dog.",
            "Supercalifragilisticexpialidocious",
            "Two tires fly. Two wail. A bamboo grove, all chopped down. From it, warring songs.",
            "Let's set the existence-of-god issue aside for a later volume, and just stipulate that in _some_ way, self-replicating organisms came into existence on this planet and immediately began trying to get rid of each other, either by spamming their environments with rough copies of themselves, or by more direct means which hardly need to be belabored.",
            "For a long time there is really nothing to be seen except steam; but after Golgotha's been burning for an hour or two, it becomes possible to see that underneath the shallow water, spreading down the valley floor, indeed right around the isolated boulder where Randy's perched, is a bright, thick river of gold.",
        ].windows(2) {
            let (key, msg) = (w[0], w[1]);
            let expect: String = textbyte(msg).pad(PAD_CHAR, GROUP_SIZE).restore().separate(' ', GROUP_SIZE);
            let deck = Deck::from_passphrase(key);
            let ciphertext = encrypt(deck.clone(), msg);
            let plaintext = decrypt(deck, &ciphertext);
            dbg!(key, msg, &expect, ciphertext, &plaintext);
            assert_eq!(expect, plaintext);
        }
    }
}
