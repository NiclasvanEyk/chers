wasm:
    wasm-pack build --target web chers

play:
    cargo run --bin=chers_cli

web:
    cd chers_web && bun run dev
