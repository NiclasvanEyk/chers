wasm-dev:
    cargo build --package=chers --target=wasm32-unknown-unknown
    wasm-bindgen target/wasm32-unknown-unknown/debug/chers.wasm --target=web --debug --out-dir=chers_web/generated/chers

wasm-release:
    cargo build --package=chers --target=wasm32-unknown-unknown --release
    wasm-bindgen target/wasm32-unknown-unknown/release/chers.wasm --target=web --out-dir=chers_web/generated/chers

server-ts:
    rm -rf chers_web/generated/chers_server_api
    mkdir -p chers_web/generated/chers_server_api
    # ts-rs exports relative to crate root, so use absolute path
    TS_RS_EXPORT_DIR={{justfile_directory()}}/chers_web/generated/chers_server_api cargo test --package=chers_server_api

# Start backend server with debug logging on port 8000
server-dev:
    #!/usr/bin/env bash
    set -euxo pipefail
    cd chers_server
    export PORT=8000
    export RUST_LOG=chers_server=debug,axum=info,tower_http=info
    export OTEL_EXPORTER_OTLP_ENDPOINT="${OTEL_EXPORTER_OTLP_ENDPOINT:-http://127.0.0.1:18889}"
    export OTEL_TRACES_SAMPLER_ARG="${OTEL_TRACES_SAMPLER_ARG:-1.0}"
    export OTEL_SERVICE_NAME="${OTEL_SERVICE_NAME:-chers-server}"
    export SENTRY_ENVIRONMENT="${SENTRY_ENVIRONMENT:-}"
    export SENTRY_TRACES_SAMPLE_RATE="${SENTRY_TRACES_SAMPLE_RATE:-1.0}"
    cargo run

play:
    cargo run --bin=chers_cli

watch-rust-tests:
    cargo watch --clear --quiet --exec 'nextest run --success-output=never --status-level=fail --hide-progress-bar'

web-dev: wasm-dev server-ts
    pnpm install && pnpm --filter chers_web run dev

web-release: wasm-release server-ts
    pnpm install && pnpm --filter chers_web run build

clean:
    cargo clean
    rm -rf node_modules chers_web/generated/chers*
