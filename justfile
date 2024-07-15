wasm:
    wasm-pack build --target web chers
    cargo -p chers_server_api test

play:
    cargo run --bin=chers_cli

watch-rust-tests:
    cargo watch --clear --quiet --exec 'nextest run --success-output=never --status-level=fail --hide-progress-bar'

web:
    cd chers_web && bun run dev
