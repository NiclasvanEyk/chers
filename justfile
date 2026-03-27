wasm-dev:
    cargo build --package=chers --target=wasm32-unknown-unknown
    wasm-bindgen target/wasm32-unknown-unknown/debug/chers.wasm --target=web --debug --out-dir=chers_web/generated/chers

wasm-release:
    cargo build --package=chers --target=wasm32-unknown-unknown --release
    wasm-bindgen target/wasm32-unknown-unknown/release/chers.wasm --target=web --out-dir=chers_web/generated/chers

play:
    cargo run --bin=chers_cli

watch-rust-tests:
    cargo watch --clear --quiet --exec 'nextest run --success-output=never --status-level=fail --hide-progress-bar'

web-dev: wasm-dev
    pnpm install && pnpm --filter chers_web run dev

web-release: wasm-release
    pnpm install && pnpm --filter chers_web run build

clean:
    cargo clean
    rm -rf node_modules chers_web/generated/chers*
