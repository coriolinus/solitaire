use itertools::Itertools;
use std::iter::FromIterator;

/// Convert a text input into a numeric stream from 1..26 according to its chars.
///
/// ASCII letters are uppercased, then assigned `A==1 .. Z==26`. All other chars
/// are discarded.
pub fn textbyte(text: &str) -> impl '_ + Iterator<Item = u8> {
    text.chars()
        .filter(char::is_ascii_alphabetic)
        .map(|c| (c.to_ascii_uppercase() as u8) - b'A' + 1)
}

pub type Padded<'a, T> = Box<dyn 'a + Iterator<Item = T>>;
pub trait Pad<'a, T>
where
    T: Copy,
{
    /// Ensure that a stream has a length of a multiple of `group_size`.
    ///
    /// If `iter` ends at a length which is not a multiple of `group_size`,
    /// instances of `padding` are copied into the stream until the length
    /// is correct.
    ///
    /// This is a fused iterator.
    fn pad(self, padding: T, group_size: usize) -> Padded<'a, T>;
}

impl<'a, I, T> Pad<'a, T> for I
where
    I: IntoIterator<Item = T>,
    <I as IntoIterator>::IntoIter: 'a,
    T: 'a + Copy,
{
    fn pad(self, padding: T, group_size: usize) -> Padded<'a, T> {
        use itertools::EitherOrBoth::*;
        Box::new(
            self.into_iter()
                .fuse()
                .zip_longest(std::iter::repeat(padding))
                .enumerate()
                .take_while(move |(idx, eob)| match eob {
                    Left(_) => unreachable!(),
                    Both(_, _) => true,
                    Right(_) => idx % group_size != 0,
                })
                .map(|(_, eob)| match eob {
                    Left(_) => unreachable!(),
                    Both(b, _) => b,
                    Right(b) => b,
                }),
        )
    }
}

pub type Restored<'a> = Box<dyn 'a + Iterator<Item = char>>;
pub trait Restore<'a> {
    /// Restore a stream of bytes into a stream of characters.
    ///
    /// Assumes the mapping is `1==A .. 26==Z`;
    ///
    /// This is a fused iterator.
    fn restore(self) -> Restored<'a>;
}

impl<'a, I> Restore<'a> for I
where
    I: IntoIterator<Item = u8>,
    <I as IntoIterator>::IntoIter: 'a,
{
    fn restore(self) -> Restored<'a> {
        Box::new(
            self.into_iter()
                .fuse()
                .map(|b| (((b - 1) % 26) + b'A') as char),
        )
    }
}

pub trait Separate<'a, I, T, O>
where
    I: IntoIterator<Item = T>,
    T: Copy,
    O: FromIterator<T>,
{
    /// Separate a stream into groups, inserting a copy of T between each.
    /// Then collect it into a String.
    ///
    /// This is a fused iterator.
    fn separate(self, group_sep: T, group_size: usize) -> O;
}

impl<'a, I, T, O> Separate<'a, I, T, O> for I
where
    I: 'a + IntoIterator<Item = T>,
    <I as IntoIterator>::IntoIter: 'a,
    T: 'a + Copy + PartialEq,
    O: FromIterator<T>,
{
    fn separate(self, group_sep: T, group_size: usize) -> O {
        self.into_iter()
            .fuse()
            .chunks(group_size)
            .into_iter()
            .map(|chunk| {
                let d: Box<dyn Iterator<Item = T>> = Box::new(chunk);
                d
            })
            .interleave_shortest(std::iter::repeat(std::iter::once(group_sep)).map(|cyc| {
                let d: Box<dyn Iterator<Item = T>> = Box::new(cyc);
                d
            }))
            .flatten()
            .with_position()
            .filter_map(move |pos| {
                use itertools::Position::*;
                match pos {
                    Only(c) => Some(c),
                    First(c) => Some(c),
                    Middle(c) => Some(c),
                    Last(c) if c != group_sep => Some(c),
                    _ => None,
                }
            })
            .collect()
    }
}

pub mod prelude {
    pub use super::textbyte;
    pub use super::Pad;
    pub use super::Restore;
    pub use super::Separate;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GROUP_SIZE, PAD_CHAR};

    fn convert_text_impl(msg: &str, expect: &[u8]) {
        let have: Vec<u8> = textbyte(msg).collect();
        assert_eq!(have, expect);
    }

    #[test]
    fn test_convert_text() {
        convert_text_impl("abc", &[1, 2, 3]);
        convert_text_impl("xyz", &[24, 25, 26]);
        convert_text_impl("abc def", &[1, 2, 3, 4, 5, 6]);
        convert_text_impl("xyz.fed", &[24, 25, 26, 6, 5, 4]);
    }

    fn padding_impl(msg: &str, expect_len: usize) {
        assert_eq!(
            textbyte(msg)
                .pad(PAD_CHAR, GROUP_SIZE)
                .collect::<Vec<_>>()
                .len(),
            expect_len
        );
    }

    #[test]
    fn test_padding() {
        padding_impl("a", 5);
        padding_impl("abcde", 5);
        padding_impl(".", 0);
        padding_impl("abcdef", 10);
        padding_impl("a.b.c.d", 5);
        padding_impl("", 0);
    }

    fn padding_impl_2(msg: &str, expect: &[u8]) {
        let have: Vec<u8> = textbyte(msg).pad(PAD_CHAR, GROUP_SIZE).collect();
        assert_eq!(have, expect);
    }

    #[test]
    fn test_padding_2() {
        padding_impl_2("a", &[1, 24, 24, 24, 24]);
        padding_impl_2("abcde", &[1, 2, 3, 4, 5]);
        padding_impl_2(".", &[]);
        padding_impl_2("abcdef", &[1, 2, 3, 4, 5, 6, 24, 24, 24, 24]);
        padding_impl_2("a.b.c.d", &[1, 2, 3, 4, 24]);
        padding_impl_2("", &[]);
    }

    fn reverse_impl(msg: &str) {
        let msg = &msg.to_string().to_uppercase();
        assert_eq!(&textbyte(msg).restore().collect::<String>(), msg,);
    }

    #[test]
    fn test_reverse() {
        reverse_impl("abc");
        reverse_impl("zyx");
        reverse_impl("abcdefghijklmnopqrstuvwxyz");
        reverse_impl("thequickbrownfoxjumpedoverthelazydog");
        reverse_impl("abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz");
    }

    #[test]
    fn test_separate() {
        for (msg, expect) in &[
            ("abc", "abc"),
            ("zyx", "zyx"),
            (
                "abcdefghijklmnopqrstuvwxyz",
                "abcde fghij klmno pqrst uvwxy z",
            ),
            (
                "thequickbrownfoxjumpedoverthelazydog",
                "thequ ickbr ownfo xjump edove rthel azydo g",
            ),
            (
                "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz",
                "abcde fghij klmno pqrst uvwxy zabcd efghi jklmn opqrs tuvwx yz",
            ),
        ] {
            let got: String = msg.chars().separate(' ', 5);
            assert_eq!(&got, expect,);
        }
    }
}
