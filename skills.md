# skills — signal-persona-system

*Per-repo agent guide.*

## Checkpoint — read before editing

Before changing code in this repo, read:

- `~/primary/skills/contract-repo.md`
- `~/primary/skills/architecture-editor.md`
- `~/primary/skills/architectural-truth-tests.md`
- `~/primary/skills/push-not-pull.md` (this channel IS
  the push-fed substrate the router subscribes to)
- `~/primary/skills/nix-discipline.md`
- this repo's `ARCHITECTURE.md`
- the consumers' `ARCHITECTURE.md` files
  (`persona-system/`, `persona-router/`)

If your change adds a new subscription kind or observation
event, edit `src/lib.rs` first, then push, then update the
consumers.

## What this repo owns

- `SystemTarget` (typed enum over backend windows;
  currently NiriWindow only).
- The closed `SystemRequest` enum (subscription +
  observation requests from the router).
- The closed `SystemEvent` enum (focus, window-lifecycle,
  and subscription events from the system).
- `SubscriptionKind`.
- The `Frame` type alias = `signal_core::Frame<SystemRequest, SystemEvent>`.
- The wire-form round-trip tests in `tests/round_trip.rs`.

## What this repo does not own

- The Niri adapter (lives in `persona-system`).
- Terminal prompt cleanliness, input gates, and write-injection
  safety (live in `signal-persona-terminal` and `persona-terminal`).
- Transport (UDS path, reconnect, timeouts) — per consumer.
- Subscription lifetime / accounting — that's
  `persona-system`'s actor.
