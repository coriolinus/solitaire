use itertools::Itertools;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::ptr::NonNull;

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

// this whole mess is kind of a nightmare: we can't just run it as a normal
// iterator combinator which can `impl Iterator` because `into_chunks` _has_ to
// live in separate memory, for just as long as the `separated` struct which
// references it. The only reasonable way to ensure that, while _looking like_ a
// combinator, is to implement this as a self-referential struct. Luckily, that
// sort of thing is at least possible now.
type SepPtr<'a, T> = NonNull<dyn 'a + Iterator<Item = T>>;
type IntoChunks<I> = itertools::IntoChunks<std::iter::Fuse<<I as IntoIterator>::IntoIter>>;
pub struct Separated<'a, I, T>
where
    I: IntoIterator<Item = T>,
{
    into_chunks: IntoChunks<I>,
    separated: SepPtr<'a, T>,
    _pin: PhantomPinned,
}

impl<'sep, 'a: 'sep, I, T> Separated<'a, I, T>
where
    I: 'a + IntoIterator<Item = T>,
    <I as IntoIterator>::IntoIter: 'a,
    T: 'a + Copy + PartialEq,
{
    // see https://doc.rust-lang.org/std/pin/index.html#example-self-referential-struct
    fn new(iter: I, group_sep: T, group_size: usize) -> Pin<Box<Self>> {
        // What we want is to create a dangling pointer at this point for
        // `separated`, because we just need a throwaway value. Unfortunately,
        // that doesn't work: `NonNull::dangling` doesn't have the `?Sized` bound
        // for its `T`, and dynamic function pointers are of course not sized.
        //
        // Therefore, we work around the issue by creating a temporary function
        // pointer to throw in there right now; it should get cleaned up at the
        // end of this scope.
        //
        // We know the unsafe block in this declaration is actually safe because
        // we use the unchecked pointer only immediately after declaring it to
        // a valid value.
        let temporary_iter: *mut _ = &mut std::iter::once(group_sep);
        let sep = Separated {
            into_chunks: iter.into_iter().fuse().chunks(group_size),
            separated: unsafe { NonNull::new_unchecked(temporary_iter) },
            _pin: PhantomPinned,
        };

        let mut boxed = Box::pin(sep);

        let separated = NonNull::from(
            &(boxed
                .into_chunks
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
                })),
        );

        // this is safe because modifying a field doesn't move the whole struct
        unsafe {
            let mut_ref: Pin<&'sep mut Self> = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).separated = separated;
        }
        boxed
    }
}

impl<'a, I, T> Iterator for Separated<'a, I, T>
where
    I: IntoIterator<Item = T>,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        // we never share this mutable reference, and it drops as soon
        // as this function goes out of scope, so we don't need to worry about
        // the possible of other immutable or mutable access while we're
        // within this block.
        unsafe { self.separated.as_mut() }.next()
    }
}

pub trait Separate<'a, I, T>
where
    I: IntoIterator<Item = T>,
    T: Copy,
{
    /// Separate a stream into groups, inserting a copy of T between each.
    ///
    /// This is a fused iterator.
    fn separate(self, group_sep: T, group_size: usize) -> Pin<Box<Separated<'a, I, T>>>;
}

impl<'a, I, T> Separate<'a, I, T> for I
where
    I: 'a + IntoIterator<Item = T>,
    <I as IntoIterator>::IntoIter: 'a,
    T: 'a + Copy + PartialEq,
{
    fn separate(self, group_sep: T, group_size: usize) -> Pin<Box<Separated<'a, I, T>>> {
        Separated::new(self, group_sep, group_size)
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

    fn test_convert_text_impl(msg: &str, expect: &[u8]) {
        let have: Vec<u8> = convert_text(msg).collect();
        assert_eq!(have, expect);
    }

    #[test]
    fn test_convert_text() {
        test_convert_text_impl("abc", &[1, 2, 3]);
        test_convert_text_impl("xyz", &[24, 25, 26]);
        test_convert_text_impl("abc def", &[1, 2, 3, 4, 5, 6]);
        test_convert_text_impl("xyz.fed", &[24, 25, 26, 6, 5, 4]);
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

    fn test_padding_impl_2(msg: &str, expect: &[u8]) {
        let have: Vec<u8> = pad_text(convert_text(msg)).collect();
        assert_eq!(have, expect);
    }

    #[test]
    fn test_padding_2() {
        test_padding_impl_2("a", &[1, 24, 24, 24, 24]);
        test_padding_impl_2("abcde", &[1, 2, 3, 4, 5]);
        test_padding_impl_2(".", &[]);
        test_padding_impl_2("abcdef", &[1, 2, 3, 4, 5, 6, 24, 24, 24, 24]);
        test_padding_impl_2("a.b.c.d", &[1, 2, 3, 4, 24]);
        test_padding_impl_2("", &[]);
    }
}
