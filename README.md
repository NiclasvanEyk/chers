# Chers

Chess implemented in Rust.
Contains:

- A core [engine](.chers/) that validates and suggests moves
- A [command line interface](.chers_cli/) that enables you to play chess inside your terminal
- A [Universal Chess Interface](./chers_uci/) implementation (WIP)

## Try it out

```shell
git clone https://github.com/NiclasvanEyk/chers
cd chers
cargo run --bin chers_cli
```

## Engine TODOs

- [x] En passant
- [ ] Pawn promition
- [ ] Castling
- [ ] Halfmove clock
- [ ] Fullmove number
- [ ] Checkmate
- [ ] Mate
- [ ] Remove this list once all items are finished

## Ideas

- Add prettier board state rendering, when the terminal supports the [kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/)
- Add an actual engine that can make moves on its own. This will be part of `chers_uci`.
