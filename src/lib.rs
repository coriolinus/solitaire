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
    use rstest::rstest;

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
    /// This test is the first example in the book, adjusted so it passes.
    fn test_example_outputs_3() {
        assert_eq!(
            &keystream(Deck::new()).take(9).collect::<Vec<_>>(),
            &[4, 49, 10, 24, 8, 51, 44, 6, 4],
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

    #[test]
    fn test_empty_key_produces_sorted_deck() {
        assert_eq!(Deck::from_passphrase(""), Deck::new(),)
    }

    /// tests from the vectors at: https://www.schneier.com/code/sol-test.txt
    #[rstest(plain, key, output, cipher,
        case(
            "AAAAAAAAAAAAAAA",
            "",
            Some(vec![
                4, 49, 10, 53, 24, 8, 51, 44, 6, 4, 33, 20, 39, 19, 34, 42,
            ]),
            "EXKYI ZSGEH UNTIQ",
        ),
        case(
            "AAAAAAAAAAAAAAA",
            "f",
            Some(vec![49, 24, 8, 46, 16, 1, 12, 33, 10, 10, 9, 27, 4, 32, 24]),
            "XYIUQ BMHKK JBEGY",
        ),
        case(
            "AAAAAAAAAAAAAAA",
            "fo",
            Some(vec![
                19, 46, 9, 24, 12, 1, 4, 43, 11, 32, 23, 39, 29, 34, 22,
            ]),
            "TUJYM BERLG XNDIW",
        ),
        case(
            "AAAAAAAAAAAAAAA",
            "foo",
            Some(vec![
                8, 19, 7, 25, 20, 53, 9, 8, 22, 32, 43, 5, 26, 17, 53, 38, 48,
            ]),
            "ITHZU JIWGR FARMW",
        ),
        case(
            "AAAAAAAAAAAAAAA",
            "a",
            Some(vec![
                49, 14, 3, 26, 11, 32, 18, 2, 46, 37, 34, 42, 13, 18, 28,
            ]),
            "XODAL GSCUL IQNSC",
        ),
        case(
            "AAAAAAAAAAAAAAA",
            "aa",
            Some(vec![14, 7, 32, 22, 38, 23, 23, 2, 26, 8, 12, 2, 34, 16, 15]),
            "OHGWM XXCAI MCIQP",
        ),
        case(
            "AAAAAAAAAAAAAAA",
            "aaa",
            Some(vec![
                3, 28, 18, 42, 24, 33, 1, 16, 51, 53, 39, 6, 29, 43, 46, 45,
            ]),
            "DCSQY HBQZN GDRUT",
        ),
        case(
            "AAAAAAAAAAAAAAA",
            "b",
            Some(vec![
                49, 16, 4, 30, 12, 40, 8, 19, 37, 25, 47, 29, 18, 16, 18,
            ]),
            "XQEEM OITLZ VDSQS",
        ),
        case(
            "AAAAAAAAAAAAAAA",
            "bc",
            Some(vec![
                16, 13, 32, 17, 10, 42, 34, 7, 2, 37, 6, 48, 44, 28, 53, 4,
            ]),
            "QNGRK QIHCL GWSCE",
        ),
        case(
            "AAAAAAAAAAAAAAA",
            "bcd",
            Some(vec![
                5, 38, 20, 27, 50, 1, 38, 26, 49, 33, 39, 42, 49, 2, 35,
            ]),
            "FMUBY BMAXH NQXCJ",
        ),
        case(
            "AAAAAAAAAAAAAAAAAAAAAAAAA",
            "cryptonomicon",
            None,
            "SUGSR SXSWQ RMXOH IPBFP XARYQ",
        ),
        case("SOLITAIRE", "cryptonomicon", None, "KIRAK SFJAN"),
    )]
    fn vectors(plain: &str, key: &str, output: Option<Vec<u8>>, cipher: &str) {
        dbg!(key);
        let deck = Deck::from_passphrase(key);
        if let Some(mut output) = output {
            // the output vectors include the jokers, which are excluded from
            // keystream's output
            output.retain(|v| *v < 53);
            assert_eq!(
                keystream(deck.clone())
                    .take(output.len())
                    .collect::<Vec<_>>(),
                output,
            )
        }
        assert_eq!(&encrypt(deck, plain), cipher,)
    }
}
