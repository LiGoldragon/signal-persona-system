//! Signal contract — `persona-system` → `persona-router`.
//!
//! Read this file as the public interface of the OS-facts
//! channel. The channel carries:
//!
//! - **Subscription requests** from the router (start /
//!   stop watching a target's focus or input-buffer state).
//! - **One-shot observation requests** from the router
//!   (current focus state right now, no subscription).
//! - **Observation events** from `persona-system` (focus
//!   changes, input-buffer changes, target lifecycle).
//!
//! The channel is **bidirectional**: the router initiates
//! subscriptions; the system pushes observation events back
//! over the same channel after subscriptions are accepted.
//! Per `~/primary/skills/push-not-pull.md`, the system
//! pushes; the router never polls.
//!
//! See `ARCHITECTURE.md` for the channel's role and
//! boundaries; `~/primary/reports/designer/72-harmonized-implementation-plan.md`
//! §6 for the contract-creation discipline.

use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use signal_core::signal_channel;

// ─── Target identity ──────────────────────────────────────

/// A typed identifier for a window-shaped OS surface the
/// router cares about. Currently only Niri windows; future
/// backends (Mac, Hyprland, etc.) add variants through a
/// coordinated schema upgrade because this closed enum rejects
/// unknown variants at decode time.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemTarget {
    NiriWindow(NiriWindowId),
}

/// Niri's typed window id (a u64 newtype).
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NiriWindowId(u64);

impl NiriWindowId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

impl SystemTarget {
    pub const fn niri_window(window_id: u64) -> Self {
        Self::NiriWindow(NiriWindowId::new(window_id))
    }
}

// ─── Subscription requests (router → system) ──────────────

/// Monotonic observation counter minted by `persona-system`.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObservationGeneration(u64);

impl ObservationGeneration {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn into_u64(self) -> u64 {
        self.0
    }
}

/// Subscribe to focus events for `target`. The system
/// replies with an `Accepted` event and then pushes
/// `FocusObservation` events whenever focus changes.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct FocusSubscription {
    pub target: SystemTarget,
}

/// Stop receiving focus events for `target`.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct FocusUnsubscription {
    pub target: SystemTarget,
}

/// One-shot: what is the focus state for `target` *right
/// now*? Reply is a single `FocusObservation` event; no
/// subscription established.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct FocusSnapshot {
    pub target: SystemTarget,
}

/// Subscribe to input-buffer events for `target`. The
/// system pushes `InputBufferObservation` events whenever
/// the prompt buffer transitions between Empty / Occupied /
/// Unknown.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct InputBufferSubscription {
    pub target: SystemTarget,
}

/// Stop receiving input-buffer events for `target`.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct InputBufferUnsubscription {
    pub target: SystemTarget,
}

/// One-shot: what's in the input buffer for `target` right
/// now?
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct InputBufferSnapshot {
    pub target: SystemTarget,
}

// ─── Observation events (system → router) ─────────────────

/// Focus changed (or current state, for a one-shot `FocusSnapshot`).
/// `generation` is a monotonic counter the system mints; the
/// router uses it to discard stale events when subscriptions
/// race.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusObservation {
    pub target: SystemTarget,
    pub focused: bool,
    pub generation: ObservationGeneration,
}

/// Input-buffer state changed (or current state, for
/// one-shot `InputBufferSnapshot`).
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct InputBufferObservation {
    pub target: SystemTarget,
    pub state: InputBufferState,
    pub generation: ObservationGeneration,
}

/// What's in a target's input buffer.
///
/// `Unknown` is the safe default: the router treats it the
/// same as `Occupied` (per the safety property — never
/// inject when uncertain).
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum InputBufferState {
    Empty,
    Occupied,
    Unknown,
}

/// The target window has gone away (closed by user, killed,
/// etc.). The system stops emitting events for it; existing
/// subscriptions on that target are implicitly cancelled.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowClosed {
    pub target: SystemTarget,
}

/// Subscription was accepted; events of the named kind will
/// follow.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubscriptionAccepted {
    pub target: SystemTarget,
    pub kind: SubscriptionKind,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscriptionKind {
    Focus,
    InputBuffer,
}

/// The system can't observe the named target — it doesn't
/// exist (yet, or any more), or the system has no backend
/// for it.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObservationTargetMissing {
    pub target: SystemTarget,
}

// ─── Channel declaration ───────────────────────────────────

signal_channel! {
    request SystemRequest {
        FocusSubscription(FocusSubscription),
        FocusUnsubscription(FocusUnsubscription),
        FocusSnapshot(FocusSnapshot),
        InputBufferSubscription(InputBufferSubscription),
        InputBufferUnsubscription(InputBufferUnsubscription),
        InputBufferSnapshot(InputBufferSnapshot),
    }
    reply SystemEvent {
        FocusObservation(FocusObservation),
        InputBufferObservation(InputBufferObservation),
        WindowClosed(WindowClosed),
        SubscriptionAccepted(SubscriptionAccepted),
        ObservationTargetMissing(ObservationTargetMissing),
    }
}
