# Chers Server Migration Plan

Replace the old `chers_server/` with `another-chess-server/` and deploy on Fly.io.

- [x] **1. Remove old server & rename new server**
  - [x] Delete `chers_server/` directory
  - [x] Rename `another-chess-server/` → `chers_server/`
  - [x] Update workspace `Cargo.toml`: change `"another-chess-server"` member to `"chers_server"`
  - [x] Update `chers_server/Cargo.toml`: set `name = "chers_server"`, keep `edition = "2024"`

- [ ] **2. Update Rust code references**
  - [ ] `chers_server/src/main.rs`: replace `another_chess_server::` → `chers_server::`
  - [ ] `chers_server/src/lib.rs`: update any references
  - [ ] `chers_server/src/config.rs`: update references in feature-gated blocks
  - [ ] `chers_server/src/server/mod.rs`: update references
  - [ ] `chers_server/src/server/ws.rs`: update references
  - [ ] `chers_server/src/room/storage/`: update references
  - [ ] `chers_server/src/communication/`: update references in all sub-modules
  - [ ] `chers_server/src/actor/`: update references in all sub-modules
  - [ ] `chers_server/tests/`: update test imports if needed
  - [ ] Run `cargo check` to verify no broken references remain

- [ ] **3. Add `PORT` env var fallback to `main.rs` (Option A)**
  - [ ] Modify `main.rs` to read `PORT` first, fall back to `CHERS_ADDR`, then default `0.0.0.0:8000`
  - [ ] Verify with `cargo check`

- [ ] **4. Update `justfile` for the new server**
  - [ ] Remove `chers-static` and `chers-static-dev` recipes (no `bundle-frontend` feature)
  - [ ] Update `server-dev` recipe to work with the new server (uses `PORT` + correct tracing env vars)

- [ ] **5. Update `Dockerfile` for Fly deployment**
  - [ ] Change build command: `cargo build --release --features "nats,otel" --bin chers_server`
    *(NATS + OTel enabled, Redis excluded)*
  - [ ] Ensure `PORT` env var flows through correctly (already handled by step 3)

- [ ] **6. Update `fly.toml`**
  - [ ] Set env vars for NATS connection (e.g. `CHERS_NATS_URL`)
  - [ ] Set env vars for OTel (if needed)
  - [ ] Set `CHERS_ADDR` or rely on `PORT` fallback (step 3)
  - [ ] Remove `SENTRY_ENVIRONMENT` (no Sentry dependency in new server)
  - [ ] Verify internal port matches (8080)

- [ ] **7. Configure Fly secrets**
  - [ ] Add NATS URL/credentials as Fly secrets
  - [ ] Add any other secrets (OTEL endpoint keys, etc.)

- [ ] **8. Deploy to Fly**
  - [ ] `fly deploy`
  - [ ] Verify `/health` responds with `"OK"`
  - [ ] Test a full game lifecycle via the frontend

- [ ] **9. Housekeeping**
  - [ ] Update root `AGENTS.md` if it references old server docs paths
  - [ ] Update `DEPLOYMENT.md` to reflect new deployment model
  - [ ] Remove any stale references to `chers_static` or `bundle-frontend`
