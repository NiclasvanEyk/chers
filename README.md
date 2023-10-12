# Chers

Chess implemented in Rust.
Contains:

- A core [engine](./chers/) that validates and suggests moves
- A [command line interface](./chers_cli/) that enables you to play chess inside your terminal
- A [web interface](./chers_web/) that re-uses the core engine compiled to Webassembly to run chess in your browser

## Try it out

Run

```shell
cd /tmp
git clone https://github.com/NiclasvanEyk/chers
cd chers
cargo run --bin chers_cli
```

to get up and running in the terminal (requires Rust to be installed locally) or visit https://chers.niclasve.me and try out the web version.

## Engine TODOs

- [x] En passant
- [x] Pawn promotion
- [ ] Castling
- [ ] Halfmove clock
- [ ] Fullmove number
- [x] Checkmate
- [x] Mate
- [ ] Remove this list once all items are finished

## Ideas

- Add prettier board state rendering, when the terminal supports the [kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/)
