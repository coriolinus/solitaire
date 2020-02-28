# Solitaire

Bruce Schneier's _Solitaire_ algorithm, as described in Neil Stephenson's _Cryptonomicon_.

## Overview

Solitaire is a crypto algorithm which uses permutations of a fairly standard deck of cards to generate a stream of pseudo-random alphabetical characters, which can be added to

### Key

The initial permutation of the deck is the message key. This project includes some support for generating an initial permutation of the deck, but there's a lot of entropy in a deck's permutation, more than is easily available on most computers. It may be better to generate your own permutation and feed it in.

## Implementation details

This implementation uses a standard array to store the cards. I considered basing this on `alloc::collections::LinkedList` and its `CursorMut` implementation instead, because it seemed like fun and could make cut operations much more performant. I chose not to for two reasons:

- Searching through a slice is much faster than searching through a linked list: LLs trash the processor cache, where slices utilize it efficiently. Searching is a common operation in this algorithm.
- There are 54 cards in a deck, and each card's representation should be maybe 2 bytes. Structs this size are declared `Copy` all the time, so there's no point trying to avoid just rewriting the whole array as necessary.

## Testing

Because const generics are not yet (early 2020) a thing in Rust, the size of a deck is hardcoded as a constant. However, for testing purposes, it's much simpler and clearer when the deck is much smaller and made of simple integers, not cards. Therefore, certain basic
tests are behind a feature gate. To run all tests, do

```sh
cargo test --features small-deck-tests && cargo test
```
