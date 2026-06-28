# Autonomous on-chain payment agent (Solana, Rust) — Build Plan

> Self-contained build plan so this work can be resumed on any machine. Built with
> "loop engineering": each phase ends in an objective pass/fail gate so an iterative
> build loop (`/loop`) has a clear signal before advancing.

## Resuming on another machine

1. Make sure this file is committed and pushed: `git add docs/agent-build-plan.md && git commit -m "docs: agent build plan" && git push` (then `git pull` on the other laptop).
2. Open the repo in Claude Code on the other laptop.
3. Recreate the task list from the **Build order** checklist below (the in-app task list is session-local and does not sync).
4. Drive the build with the loop, e.g. per phase:
   `/loop implement the next unchecked phase in docs/agent-build-plan.md; run the phase gate (cargo fmt --check && cargo check && cargo test && cargo clippy -- -D warnings, plus any phase-specific check); only when green, mark the phase done in this file and stop`
   Omit the interval so the model self-paces, or set one (e.g. `/loop 10m ...`).
5. Prereqs on the other machine: Rust toolchain, Docker (for Postgres), Node/Expo (for the frontend phase), Solana CLI (devnet keys/airdrop), and an `ANTHROPIC_API_KEY`.

---

## Context

We are building an **autonomous payment agent** for cross-border settlement on Solana. The agent runs a continuous **observe → decide → act → verify** loop: it watches a treasury wallet for inbound USDC, asks **Claude** how to settle it against stored instructions, validates the decision against hard policy guardrails, then **signs and sends** the payout transactions itself, finally recording and verifying confirmation.

This is a genuinely new capability for the codebase. Today the backend is **read-only**: it tracks wallets, parses transactions, and builds *unsigned* transactions that the *frontend* signs. The agent inverts this — it must custody a key and sign autonomously. A lot is still reusable.

**Scoping decisions (confirmed):**
- Decision engine: **Claude API in-loop** (Anthropic Messages API + tool use, structured decision).
- Network: **Devnet POC** with a real agent keypair, airdropped SOL + test USDC. Zero financial risk.
- Flow: **Receive → route → payout** (inbound USDC matched to a settlement instruction, split with fees, paid to recipients, receipt recorded).
- Structure: **new binary in the existing crate** (`src/bin/agent.rs` + an `agent/` library module), reusing `SolanaClient`, `DepositService`, DB, config, error types.
- Frontend: **new "Agent" tab in the existing Expo app** with a live activity feed, settlement instructions, treasury balance, and a "simulate inbound payment" button.

## Reuse (do not re-implement)

| Need | Reuse | Location |
|---|---|---|
| Read balances / signatures / parse txs | `SolanaClient` (`get_usdc_balance`, `get_token_balance`, `get_signatures`, `sync_wallet_transactions`) | `backend/src/services/solana.rs` |
| Recent blockhash | `DepositService::get_recent_blockhash` | `backend/src/services/deposit/rpc.rs:106` |
| Send signed tx | `DepositService::send_raw_transaction` | `backend/src/services/deposit/rpc.rs:140` |
| Confirm tx | `DepositService::confirm_transaction` | `backend/src/services/deposit/rpc.rs:185` |
| Idempotent ATA instruction (reference) | `build_create_ata_idempotent_instruction` (private; prefer `spl-associated-token-account` crate) | `backend/src/services/deposit/mod.rs:75` |
| Background loop + graceful shutdown pattern | `SyncService::start_background_sync` (`Arc<AtomicBool>`, tokio loop) | `backend/src/services/sync.rs:52` |
| Settlement receipt notifications | `WebhookService::notify_payment_received` | `backend/src/services/webhook.rs` |
| Config / errors / DB pool | `Config`, `AppError`, `Database` | `backend/src/config.rs`, `error.rs`, `db/mod.rs` |
| Frontend fetch / list / wallet UI patterns | existing screens & components | `frontend/app/(tabs)/`, `frontend/components/` |

## Architecture

```
src/lib.rs                  # NEW: expose modules + AppState so both binaries share code
src/main.rs                 # slimmed: uses stablecoin_pay::* (API server, default-run)
src/bin/agent.rs            # NEW: agent process entrypoint; --once flag for one iteration
src/agent/
  mod.rs                    # AgentService + start_loop() (mirrors SyncService) + run_once()
  signer.rs                 # load treasury Keypair from env; devnet guard
  context.rs                # build the observation context for Claude
  claude.rs                 # Anthropic Messages API client w/ tool-use -> Decision
  policy.rs                 # hard guardrails (limits, allowlist, balance, sanity)
  executor.rs               # build USDC transfer instr, sign, send, confirm  <-- shared lib
  types.rs                  # Decision, AgentAction, SettlementInstruction, Payout, etc.
```

**Crate refactor (Phase 1, do first):** `src/main.rs` and `src/bin/agent.rs` are separate binary crates and cannot share modules directly. Add `src/lib.rs` (crate name `stablecoin_pay`) that declares `pub mod {api, config, db, domain, error, repository, services, agent}` and defines `AppState`. Slim `main.rs` to `use stablecoin_pay::*`. Set `default-run = "stablecoin-pay"` in `[package]` so `cargo run` still starts the API; the agent runs via `cargo run --bin agent`. Handlers already reference `crate::AppState`, which resolves correctly once `AppState` lives at the lib root.

### Language: all-Rust runtime loop
The entire runtime loop is **Rust**, single binary, single process, reusing the existing backend. The Claude integration is just an Anthropic Messages API call (tool-use is JSON over HTTP via `reqwest`, already a dependency), fully isolated behind `agent/claude.rs`. No Agent SDK is needed for a bounded, fixed-tool decision. Isolating it in one module means a future swap to an SDK-based orchestrator (TS/Python) touches only `claude.rs`.

### Core invariant: model proposes, Rust disposes
Reasoning may be exploratory, but **action is never a tool the model loops on**. Claude emits exactly **one structured decision** per inbound payment; the Rust **policy gate** validates it; the Rust **executor** signs and sends. There is no `send_transaction` tool exposed to the model, ever. This keeps settlement deterministic, idempotent (one inbound processed once), auditable (one decision artifact per payment), and bounded in cost/latency.

### The loop (in `agent/mod.rs`, modeled on `sync.rs`)
1. **Observe** (`context.rs`): detect new inbound `receive` USDC txs to the treasury (reuse `sync_wallet_transactions` + a "last processed signature" cursor table). Build a context object: treasury balance, the inbound payment, open `settlement_instructions`, recent decisions, estimated fee.
2. **Decide** (`claude.rs`): call Claude (`claude-sonnet-4-6` default, configurable) with the context and a **decision tool schema** exposing only outcomes: `pay_recipients([{address, amount}])`, `hold(reason)`, `alert(reason)`. Force tool use; parse the validated tool input into a `Decision { action, reasoning, recipients }`. Persist the raw reasoning for the UI. **v1 is single-shot** (no tool round-trips before the decision); the bounded read-only tool phase is added later (Phase 8).
3. **Validate** (`policy.rs`): reject if any recipient not on allowlist (when configured), per-tx amount > `MAX_PAYOUT_USDC`, cumulative day > `DAILY_LIMIT_USDC`, sum(payouts) > available balance, or recipient count > cap. A rejected decision becomes a `hold`/`alert`, never executes.
4. **Act** (`executor.rs`): for each approved payout build an SPL `transfer_checked` instruction (+ idempotent ATA create for recipient), fetch blockhash, build `Message`/`Transaction`, **sign with the treasury `Keypair`**, base64-encode, `send_raw_transaction`, then `confirm_transaction`.
5. **Verify + record**: write `agent_decisions` and `agent_payouts` rows with statuses and signatures; optionally fire `WebhookService` settlement receipt.

### Custody / safety (devnet)
- Treasury secret loaded from `AGENT_KEYPAIR` (base58 secret key) or a keypair file path. **Never logged.**
- `signer.rs` refuses to run if the configured RPC/cluster is not devnet unless `AGENT_ALLOW_MAINNET=true` is explicitly set. Default config (`config.rs`) currently prefers Helius mainnet, so the agent path must require an explicit devnet `SOLANA_RPC_URL`.
- Policy limits are enforced in Rust **after** Claude, so the LLM can never move funds outside the envelope.

## Database (new migrations under `backend/migrations/`)

- `007_create_settlement_instructions.sql` — `id`, `label`, `match_kind` (e.g. `next_inbound` | `from_sender`), `match_value` (nullable sender address), `token`, `recipients JSONB` (`[{address, share_bps | fixed_amount}]`), `fee_bps`, `status`, `created_at`.
- `008_create_agent_decisions.sql` — `id`, `inbound_signature`, `instruction_id` (nullable), `context JSONB`, `action`, `reasoning TEXT`, `model`, `status` (`executed`|`held`|`rejected`|`failed`), `created_at`.
- `009_create_agent_payouts.sql` — `id`, `decision_id` FK, `recipient`, `amount DECIMAL(20,6)`, `signature` (nullable), `status` (`pending`|`confirmed`|`failed`), `error`, `created_at`.
- `010_create_agent_cursor.sql` — single-row cursor: `treasury_address`, `last_signature`, to avoid reprocessing inbound txs.

Add matching repository modules in `backend/src/repository/` following the existing `*_repo.rs` style (static methods taking `&PgPool`).

## API additions (existing API process — `backend/src/api/handlers/agent.rs`, wired in `api/mod.rs`)

- `GET  /agent/state` — treasury balance + cursor + summary counts.
- `GET  /agent/decisions` — recent decisions with reasoning + nested payouts (feeds the activity feed).
- `GET  /agent/instructions` / `POST /agent/instructions` — list / create settlement instructions.
- `POST /agent/simulate-inbound` — for demos: uses `executor` with a separate funded `FAUCET_KEYPAIR` to send N test USDC to the treasury, kicking off the next loop iteration.

## Frontend (`frontend/app/(tabs)/agent.tsx` + small components)

- New "Agent" tab. Sections: **Treasury** balance, **Settlement instructions** list (+ create form), **Activity feed** (each decision: inbound, Claude's action + reasoning, payout rows with clickable signatures/status), and a **Simulate inbound payment** button calling `/agent/simulate-inbound`.
- Reuse existing fetch/list patterns and types (`frontend/types/index.ts`). Poll `/agent/state` + `/agent/decisions` on an interval / focus effect (pattern already used elsewhere).

## Config / dependencies

- `backend/Cargo.toml`: add `spl-token` and `spl-associated-token-account` (versions compatible with `solana-sdk = "2"`), and `bs58` for keypair decoding. `solana-sdk` already provides `Keypair`/`Message`/`Transaction`/signing. Add `default-run = "stablecoin-pay"`. **Build risk:** SPL crate ↔ solana-sdk v2 version alignment — pin carefully and `cargo check` early.
- New env vars (extend `Config::from_env` + `.env.example`): `ANTHROPIC_API_KEY`, `ANTHROPIC_MODEL` (default `claude-sonnet-4-6`), `AGENT_KEYPAIR`, `FAUCET_KEYPAIR`, `TREASURY_ADDRESS`, `MAX_PAYOUT_USDC`, `DAILY_LIMIT_USDC`, `AGENT_LOOP_INTERVAL_SECS`, `AGENT_ALLOW_MAINNET` (default false). Devnet USDC mint differs from mainnet — set `USDC_MINT` to the devnet mint.

## Build method ("loop engineering")

Every phase must end in an **objective pass/fail gate** the loop can check before advancing, otherwise the loop has no stop signal and drifts. This is distinct from the agent's *runtime* observe→decide→act loop above; here "loop" means the development cycle.

**Standard gate (every phase):** `cargo fmt --check` → `cargo check` → `cargo test` → `cargo clippy -- -D warnings` all green (run in `backend/`).

**Enabler:** add a `--once` flag to `bin/agent.rs` that runs exactly one loop iteration and exits non-zero on failure, so chain-touching phases get a deterministic, scriptable signal instead of "it compiles."

## Build order (checklist — each phase builds on a *verified* prior one)

- [ ] **Phase 1 — Scaffold + signer.** `src/lib.rs` refactor (+ `AppState`, slim `main.rs`, `default-run`). `agent/` module (`mod.rs`, `signer.rs`, `types.rs`), `bin/agent.rs` with `--once` stub. Add Cargo deps + agent env vars. *Gate:* standard + unit test that loads a test keypair and that the mainnet guard rejects a mainnet RPC.
- [ ] **Phase 2 — Executor.** SPL `transfer_checked` + ATA create + sign + send + confirm (shared lib). *Gate:* standard + devnet integration test (`#[ignore]` by default) that sends one real transfer and asserts the recipient balance increased.
- [ ] **Phase 3 — Observe loop.** `agent_cursor` table + repo; `context.rs` detects new inbound; `mod.rs` `AgentService`/`run_once`; no actions yet (log context). *Gate:* standard + `agent --once` logs the built context for a known inbound tx and advances the cursor.
- [ ] **Phase 4 — Claude decision.** `claude.rs` tool-use, parse `Decision`, persist context+reasoning to `agent_decisions`. Single-shot. *Gate:* standard + test that a recorded context yields a well-formed `Decision` (mock or live); reasoning persisted.
- [ ] **Phase 5 — Policy guardrails + wire to executor.** `policy.rs` enforce limits/allowlist; wire decision → executor; write `agent_payouts`. *Gate:* standard + **negative test** (over-limit/out-of-allowlist decision is held/rejected and no transfer sent) and a positive `agent --once` end-to-end devnet settlement.
- [ ] **Phase 6 — API + frontend.** `handlers/agent.rs` endpoints; `settlement_instructions` table+repo; Agent tab, simulate button. *Gate:* standard + endpoints return expected JSON (curl/test); Expo web renders the activity feed; simulate button drives one loop.
- [ ] **Phase 7 — Polish.** Settlement receipts via `WebhookService`; (optional later) idle-cash yield sweep reusing Kamino/Save + DeFiLlama. *Gate:* standard + receipt webhook fires on settlement.
- [ ] **Phase 8 — Bounded read-only tool-use (the "agentic flows" showcase).** In `claude.rs` only: before the single decision, Claude may call a *capped* (≤5 turns) set of **read-only** tools (`get_counterparty_history`, `get_fee_estimate`, `check_allowlist`, `get_fx_rate`), then must commit one `Decision`. Policy + executor untouched. *Gate:* standard + a test showing the model gathers context across turns and still produces one policy-valid decision, and that the turn cap is enforced. If "explores agentic flows" is a primary goal, pull this phase earlier (right after Phase 5).

**Boundary that never moves:** no phase ever exposes an execute/send tool to the model, and no phase introduces open-ended autonomous execution. If routing complexity ever demands more, the next step is **plan/execute separation** (model produces a full settlement *plan* as one artifact → policy/human approves → Rust executes deterministically), not an autonomous write loop.

## Verification (end-to-end, devnet)

1. Create devnet keypairs for treasury + faucet; airdrop SOL (`solana airdrop`) and obtain devnet USDC to the faucet; set `.env` (devnet RPC, mint, keys, limits, `ANTHROPIC_API_KEY`).
2. `docker compose up -d`; `cd backend && cargo run` (API) and `cargo run --bin agent` (loop).
3. `POST /agent/instructions` to create a split rule (e.g. 70/30 to two devnet recipients).
4. In the Expo app's Agent tab press **Simulate inbound payment** (or `curl POST /agent/simulate-inbound`).
5. Confirm in the activity feed + logs: inbound detected → Claude decision + reasoning shown → policy passed → payouts signed/sent → confirmations; verify recipient balances changed on devnet (Solana Explorer / `get_usdc_balance`).
6. Negative test: set `MAX_PAYOUT_USDC` below the split amount and confirm the decision is **held/rejected** and **no transfer** is sent.
7. `cargo test`, `cargo clippy`, `cargo fmt` clean.

## Progress log

> Update this as you go so the next session/machine knows where things stand.

- (nothing implemented yet — Phase 1 not started)
