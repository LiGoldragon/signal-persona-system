# signal-persona-system

The Signal contract between **`persona-system`** (producer
of OS facts) and **`persona-router`** (consumer of focus
observations).

Read `src/lib.rs` for the public interface — two enums
(`SystemRequest`, `SystemEvent`) declared via the
`signal_channel!` macro. The variants ARE the messages
this channel carries.

## Quick reference

```rust
use signal_persona_system::{FocusSubscription, Frame, SystemRequest, SystemTarget};
use signal_core::{FrameBody, Request};

// Router subscribes to focus events for a Niri window
let request = SystemRequest::FocusSubscription(FocusSubscription {
    target: SystemTarget::niri_window(223),
});
let frame = Frame::new(FrameBody::Request(Request::operation(
    request.signal_verb(),
    request,
)));
let bytes = frame.encode_length_prefixed()?;
// send to persona-system's UDS
```

The system replies with `SystemEvent::SubscriptionAccepted`
followed by `SystemEvent::FocusObservation` events whenever
focus changes for the subscribed target.

`FocusSubscription` uses `Subscribe`; `FocusUnsubscription` uses `Retract`;
one-shot `FocusSnapshot` and `SystemStatusQuery` requests use `Match`.

Prompt cleanliness, input gates, and programmatic write safety are terminal
transport facts. They live in `signal-persona-terminal`, not in this system
contract.

## See also

- `ARCHITECTURE.md` — channel role + boundaries
- `~/primary/reports/designer/72-harmonized-implementation-plan.md`
  §2.1 — channel inventory
- `~/primary/reports/designer/78-convergence-with-operator-77.md`
  — work-split agreement
- `~/primary/reports/operator/67-signal-actor-messaging-gap-audit.md`
  — the safety property this channel enables
- `~/primary/skills/contract-repo.md` — contract-repo discipline
- `signal-core` — kernel that supplies `Frame`, `Request`,
  `Reply`, `signal_channel!`
