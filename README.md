# Solitaire

Bruce Schneier's _Solitaire_ algorithm, as described in Neil Stephenson's _Cryptonomicon_.

## Overview

Solitaire is a crypto algorithm which uses permutations of a fairly standard deck of cards to generate a stream of pseudo-random alphabetical characters, which can be added to an input message to encrypt it.

## Implementation details

This implementation uses a standard array to store the cards. I considered basing this on `alloc::collections::LinkedList` and its `CursorMut` implementation instead, because it seemed like fun and could make cut operations much more performant. I chose not to for two reasons:

- Searching through a slice is much faster than searching through a linked list: LLs trash the processor cache, where slices utilize it efficiently. Searching is a common operation in this algorithm.
- As each card takes 1 byte, a 54-card deck is actually pretty small. Structs this size are declared `Copy` all the time; it seems likely that it would be actually faster to just re-copy the array as required instead of messing around with list pointer manipulation optimizations.

The `textbyte` package uses a lot of trait objects, the consequence of which is that there's some indirection on function invocation for its traits. That's probably fine; in most cases, each of those functions will be called only a very few times for any given program execution, so they shouldn't represent an appreciable slowdown. While it's possible to work around that with a different design, the usage pattern is a lot uglier: nested function calls instead of call chains.

## Testing

Because const generics are not yet (early 2020) a thing in Rust, the size of a deck is hardcoded as a constant. However, for testing purposes, it's much simpler and clearer when the deck is much smaller and made of simple integers, not cards. Therefore, certain basic tests are behind a feature gate. To run all tests, do

```sh
cargo test --features small-deck-tests && cargo test
```
