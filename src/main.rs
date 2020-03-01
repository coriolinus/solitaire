use anyhow::{bail, Result};
use solitaire::{
    deck::{Deck, MaybeDeck},
    decrypt, encrypt,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "solitaire",
    about = "Bruce Schneier's Solitaire encryption algorithm."
)]
struct Opt {
    /// Only emit ASCII chars instead of unicode suit symbols
    #[structopt(short, long)]
    ascii: bool,

    #[structopt(subcommand)]
    subcommand: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(about = "shuffle a new or existing deck")]
    Shuffle {
        /// How many times to shuffle this deck.
        ///
        /// Shuffling requires entropy, and the amount of potential entropy
        /// in a truly random deck of cards is much larger than the amount which
        /// will typically be inserted by a single shuffle.
        #[structopt(short = "n", long, default_value = "7")]
        iterations: u32,

        /// Optionally specify a starting deck. Otherwise, a fresh sorted one
        /// will form the initial state.
        #[structopt(name = "deck")]
        maybe_deck: Option<MaybeDeck>,
    },
    #[structopt(about = "initialize a deck from a passphrase")]
    Passphrase { phrase: String },
    #[structopt(about = "encrypt a message")]
    Encrypt {
        #[structopt(flatten)]
        crypt_opts: CryptOptions,
    },
    #[structopt(about = "decrypt a message")]
    Decrypt {
        #[structopt(flatten)]
        crypt_opts: CryptOptions,
    },
}

#[derive(Debug, StructOpt)]
struct CryptOptions {
    /// This deck is used as the initial state.
    #[structopt(short, long, name = "deck", conflicts_with = "passphrase")]
    maybe_deck: Option<MaybeDeck>,

    /// A fresh deck is generated from this passphrase.
    #[structopt(short, long)]
    passphrase: Option<String>,

    message: String,
}

impl CryptOptions {
    fn deck(&self) -> Result<Deck> {
        if let Some(ref md) = self.maybe_deck {
            return Ok(md.clone().check()?);
        }

        if let Some(ref phrase) = self.passphrase {
            return Ok(Deck::from_passphrase(phrase));
        }

        bail!("the initial deck or a passphrase is required");
    }
}

fn main() -> Result<()> {
    use Command::*;
    let opt = Opt::from_args();

    let ascii = opt.ascii;
    let print_deck = |deck: &Deck| {
        println!(
            "{}",
            if ascii {
                deck.to_ascii_string()
            } else {
                deck.to_string()
            }
        );
    };

    match opt.subcommand {
        Shuffle {
            iterations,
            maybe_deck,
        } => {
            let mut deck = match maybe_deck {
                None => Deck::new(),
                Some(md) => md.check()?,
            };
            for _ in 0..iterations {
                deck.shuffle();
            }
            print_deck(&deck);
        }
        Passphrase { phrase } => {
            let deck = Deck::from_passphrase(&phrase);
            print_deck(&deck);
        }
        Encrypt { crypt_opts } => {
            let deck = crypt_opts.deck()?;
            println!("{}", encrypt(deck, &crypt_opts.message));
        }
        Decrypt { crypt_opts } => {
            let deck = crypt_opts.deck()?;
            println!("{}", decrypt(deck, &crypt_opts.message));
        }
    }
    Ok(())
}
