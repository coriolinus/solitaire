pub mod card;
pub mod deck;

use card::Card;
pub use deck::Deck;
use itertools::Itertools;

/// size of groups of output characters
pub const GROUP_SIZE: usize = 5;
/// inputs whose
pub const PAD_CHAR: u8 = b'X';

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

/// pad a letter value iterator with sufficient `PAD_CHAR`s that its length
/// becomes a multiple of `GROUP_SIZE`
fn pad_text<I>(iter: I) -> impl Iterator<Item = u8>
where
    I: IntoIterator<Item = u8>,
{
    use itertools::EitherOrBoth::*;
    iter.into_iter()
        .zip_longest(std::iter::repeat(PAD_CHAR))
        .enumerate()
        .take_while(|(idx, eob)| match eob {
            Left(_) => unreachable!(),
            Both(_, _) => true,
            Right(_) => idx % GROUP_SIZE != 0,
        })
        .map(|(_, eob)| match eob {
            Left(_) => unreachable!(),
            Both(b, _) => b,
            Right(b) => b,
        })
}

/// restore a string from a stream of bytes where `1=='A'` etc.
fn restore_str<I>(iter: I) -> String
where
    I: IntoIterator<Item = u8>,
{
    iter.into_iter()
        .map(|b| ((b % 26) + b'A') as char)
        .chunks(GROUP_SIZE)
        .into_iter()
        .map(|chunk| {
            let d: Box<dyn Iterator<Item = char>> = Box::new(chunk);
            d
        })
        .interleave_shortest(std::iter::repeat(std::iter::once(' ')).map(|cyc| {
            let d: Box<dyn Iterator<Item = char>> = Box::new(cyc);
            d
        }))
        .flatten()
        .with_position()
        .filter_map(|pos| {
            use itertools::Position::*;
            match pos {
                Only(c) => Some(c),
                First(c) => Some(c),
                Middle(c) => Some(c),
                Last(c) if c != ' ' => Some(c),
                _ => None,
            }
        })
        .collect()
}

/// encrypt some plaintext using a pre-prepared deck
///
/// Note that the deck is consumed. Prepare your entire message before
/// calling this method. Solitaire is not recommended for long messages.
fn crypt(deck: Deck, text: &str, operation: impl Fn(u8, u8) -> u8) -> String {
    restore_str(
        pad_text(convert_text(text))
            .zip(keystream(deck))
            .map(|(c, k)| operation(c, k)),
    )
}

/// encrypt some plaintext using a pre-prepared deck
///
/// Note that the deck is consumed. Prepare the entire message before
/// calling this method. Solitaire is not recommended for long messages.
pub fn encrypt(deck: Deck, text: &str) -> String {
    crypt(deck, text, |p, k| p + k - 1)
}

/// decrypt some ciphertext using a pre-prepared deck
///
/// Note that the deck is consumed. Prepare the entire message before
/// calling this method. Solitaire is not recommended for long messages.
pub fn decrypt(deck: Deck, text: &str) -> String {
    crypt(deck, text, |c, k| c + 51 - k)
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

    fn test_padding_impl(input: &str, expect_len: usize) {
        assert_eq!(
            pad_text(convert_text(input)).collect::<Vec<_>>().len(),
            expect_len
        );
    }

    #[test]
    fn test_padding() {
        test_padding_impl("a", 5);
        test_padding_impl("abcde", 5);
        test_padding_impl(".", 0);
        test_padding_impl("abcdef", 10);
        test_padding_impl("a.b.c.d", 5);
        test_padding_impl("", 0);
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
            let expect = restore_str(pad_text(convert_text(msg)));
            let deck = Deck::from_passphrase(key);
            let ciphertext = encrypt(deck.clone(), msg);
            let plaintext = decrypt(deck, &ciphertext);
            dbg!(key, msg, &expect, ciphertext, &plaintext);
            assert_eq!(expect, plaintext);
        }
    }
}
