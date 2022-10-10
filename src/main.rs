use anyhow::{bail, Result};
use clap::{Args, Parser, Subcommand};
use solitaire::{
    deck::{Deck, MaybeDeck},
    decrypt, encrypt,
};

#[derive(Debug, Parser)]
#[command(
    name = "solitaire",
    about = "Bruce Schneier's Solitaire encryption algorithm."
)]
struct Opt {
    /// Only emit ASCII chars instead of unicode suit symbols
    #[arg(short, long)]
    ascii: bool,

    #[command(subcommand)]
    subcommand: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(about = "shuffle a new or existing deck")]
    Shuffle {
        /// How many times to shuffle this deck.
        ///
        /// Shuffling requires entropy, and the amount of potential entropy
        /// in a truly random deck of cards is much larger than the amount which
        /// will typically be inserted by a single shuffle.
        #[arg(short = 'n', long, default_value = "7")]
        iterations: u32,

        /// Optionally specify a starting deck. Otherwise, a fresh sorted one
        /// will form the initial state.
        #[arg(name = "deck")]
        maybe_deck: Option<MaybeDeck>,
    },
    #[command(about = "initialize a deck from a passphrase")]
    Passphrase { phrase: String },
    #[command(about = "encrypt a message")]
    Encrypt {
        #[command(flatten)]
        crypt_opts: CryptOptions,
    },
    #[command(about = "decrypt a message")]
    Decrypt {
        #[command(flatten)]
        crypt_opts: CryptOptions,
    },
}

#[derive(Debug, Args)]
struct CryptOptions {
    /// This deck is used as the initial state.
    #[arg(short, long, name = "deck", conflicts_with = "passphrase")]
    maybe_deck: Option<MaybeDeck>,

    /// A fresh deck is generated from this passphrase.
    #[arg(short, long)]
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
    let opt = Opt::parse();

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
