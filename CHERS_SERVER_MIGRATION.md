# Chers Server Migration Plan

Replace the old `chers_server/` with `another-chess-server/` and deploy on Fly.io.

- [x] **1. Remove old server & rename new server**
  - [x] Delete `chers_server/` directory
  - [x] Rename `another-chess-server/` → `chers_server/`
  - [x] Update workspace `Cargo.toml`: change `"another-chess-server"` member to `"chers_server"`
  - [x] Update `chers_server/Cargo.toml`: set `name = "chers_server"`, keep `edition = "2024"`

- [x] **2. Update Rust code references**
  - [x] `chers_server/src/main.rs`: replace `another_chess_server::` → `chers_server::` (7 occurrences)
  - [x] `chers_server/tests/e2e/scenario.rs`: replace `another_chess_server::` → `chers_server::` (11 occurrences)
  - [x] `chers_server/tests/e2e/helpers.rs`: replace `another_chess_server::` → `chers_server::` (19 occurrences)
  - [x] `chers_server/src/lib.rs`, `config.rs`, `server/`, `actor/`, `room/`, `communication/`: no crate-path references, all use `crate::`
  - [x] Run `cargo check` ✓
  - [x] Run `cargo test` — 34 unit + 15 integration all pass ✓

- [x] **3. Add `PORT` env var fallback to `main.rs` (Option A)**
  - [x] Modify `main.rs` to read `PORT` first, fall back to `CHERS_ADDR`, then default `0.0.0.0:8000`
  - [x] Verify with `cargo check` ✓

- [x] **4. Update `justfile` for the new server**
  - [x] Remove `chers-static` and `chers-static-dev` recipes (no `bundle-frontend` feature)
  - [x] Update `server-dev` recipe: removed Sentry env vars (not in new server), kept OTEL config

- [x] **5. Update `Dockerfile` for Fly deployment**
  - [x] Change build command: `cargo build --release --no-default-features --features "nats,otel" --bin chers_server`
    *(NATS + OTel enabled, Redis excluded)*
  - [x] Ensure `PORT` env var flows through correctly (already handled by step 3)

- [x] **6. Update `fly.toml`**
  - [x] Set env vars for NATS connection (e.g. `CHERS_NATS_URL`)
  - [x] Set env vars for OTel (if needed)
  - [x] Set `CHERS_ADDR` or rely on `PORT` fallback (step 3)
  - [x] Remove `SENTRY_ENVIRONMENT` (no Sentry dependency in new server)
  - [x] Verify internal port matches (8080)

- [x] **7. Configure Fly secrets**
  - [x] Add NATS URL/credentials as Fly secrets
  - [x] Add any other secrets (OTEL endpoint keys, etc.)

- [ ] **8. Deploy to Fly**
  - [ ] `fly deploy`
  - [ ] Verify `/health` responds with `"OK"`
  - [ ] Test a full game lifecycle via the frontend

- [ ] **9. Housekeeping**
  - [ ] Update root `AGENTS.md` if it references old server docs paths
  - [ ] Update `DEPLOYMENT.md` to reflect new deployment model
  - [ ] Remove any stale references to `chers_static` or `bundle-frontend`
