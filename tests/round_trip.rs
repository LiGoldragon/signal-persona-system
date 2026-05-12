//! Architectural-truth round-trip tests for the
//! `signal-persona-system` channel.
//!
//! Per `~/primary/skills/architectural-truth-tests.md`,
//! each variant of both enums has a witness test that
//! proves the macro-emitted type round-trips through a
//! length-prefixed Frame.

use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode};
use signal_core::{FrameBody, Reply, Request, SemaVerb};
use signal_persona_system::{
    FocusObservation, FocusSnapshot, FocusSubscription, FocusUnsubscription, Frame,
    ObservationGeneration, ObservationTargetMissing, SubscriptionAccepted, SubscriptionKind,
    SystemEvent, SystemRequest, SystemTarget, WindowClosed,
};

const TARGET: SystemTarget = SystemTarget::niri_window(223);

fn round_trip_request(request: SystemRequest) -> SystemRequest {
    let frame = Frame::new(FrameBody::Request(Request::assert(request)));
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = Frame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        FrameBody::Request(Request::Operation { verb, payload }) => {
            assert_eq!(verb, SemaVerb::Assert);
            payload
        }
        other => panic!("expected request operation, got {other:?}"),
    }
}

fn round_trip_event(event: SystemEvent) -> SystemEvent {
    let frame = Frame::new(FrameBody::Reply(Reply::operation(event)));
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = Frame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        FrameBody::Reply(Reply::Operation(event)) => event,
        other => panic!("expected reply operation, got {other:?}"),
    }
}

fn round_trip_nota<T>(value: T, expected: &str)
where
    T: NotaEncode + NotaDecode + PartialEq + std::fmt::Debug,
{
    let mut encoder = Encoder::new();
    value.encode(&mut encoder).expect("encode nota text");
    let encoded = encoder.into_string();
    assert_eq!(encoded, expected);

    let mut decoder = Decoder::new(&encoded);
    let recovered = T::decode(&mut decoder).expect("decode nota text");
    assert_eq!(recovered, value);
}

#[test]
fn focus_subscription_round_trips() {
    let request = SystemRequest::FocusSubscription(FocusSubscription { target: TARGET });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn focus_subscription_request_round_trips_through_nota_text() {
    round_trip_nota(
        SystemRequest::FocusSubscription(FocusSubscription { target: TARGET }),
        "(FocusSubscription (NiriWindow 223))",
    );
}

#[test]
fn focus_unsubscription_round_trips() {
    let request = SystemRequest::FocusUnsubscription(FocusUnsubscription { target: TARGET });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn focus_snapshot_round_trips() {
    let request = SystemRequest::FocusSnapshot(FocusSnapshot { target: TARGET });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn focus_observation_round_trips_with_focused_true() {
    let event = SystemEvent::FocusObservation(FocusObservation {
        target: TARGET,
        focused: true,
        generation: ObservationGeneration::new(42),
    });
    let decoded = round_trip_event(event.clone());
    assert_eq!(decoded, event);
}

#[test]
fn focus_observation_round_trips_with_focused_false() {
    let event = SystemEvent::FocusObservation(FocusObservation {
        target: TARGET,
        focused: false,
        generation: ObservationGeneration::new(43),
    });
    let decoded = round_trip_event(event.clone());
    assert_eq!(decoded, event);
}

#[test]
fn window_closed_round_trips() {
    let event = SystemEvent::WindowClosed(WindowClosed { target: TARGET });
    let decoded = round_trip_event(event.clone());
    assert_eq!(decoded, event);
}

#[test]
fn subscription_accepted_round_trips_for_focus_kind() {
    let event = SystemEvent::SubscriptionAccepted(SubscriptionAccepted {
        target: TARGET,
        kind: SubscriptionKind::Focus,
    });
    let decoded = round_trip_event(event.clone());
    assert_eq!(decoded, event);
}

#[test]
fn subscription_accepted_event_round_trips_through_nota_text() {
    round_trip_nota(
        SystemEvent::SubscriptionAccepted(SubscriptionAccepted {
            target: TARGET,
            kind: SubscriptionKind::Focus,
        }),
        "(SubscriptionAccepted (NiriWindow 223) Focus)",
    );
}

#[test]
fn observation_target_missing_round_trips() {
    let event = SystemEvent::ObservationTargetMissing(ObservationTargetMissing { target: TARGET });
    let decoded = round_trip_event(event.clone());
    assert_eq!(decoded, event);
}

#[test]
fn from_impl_lifts_focus_subscription_into_request() {
    let payload = FocusSubscription { target: TARGET };
    let request: SystemRequest = payload.clone().into();
    assert_eq!(request, SystemRequest::FocusSubscription(payload));
}

#[test]
fn from_impl_lifts_focus_observation_into_event() {
    let payload = FocusObservation {
        target: TARGET,
        focused: true,
        generation: ObservationGeneration::new(1),
    };
    let event: SystemEvent = payload.into();
    assert_eq!(event, SystemEvent::FocusObservation(payload));
}

#[test]
fn system_contract_cannot_carry_terminal_prompt_gate_records() {
    let scan = DriftScan::new(env!("CARGO_MANIFEST_DIR"));

    scan.assert_absent(&[
        "InputBuffer",
        "input-buffer",
        "prompt buffer",
        "prompt-buffer",
        "gate message delivery",
        "gate deliveries",
    ]);
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DriftScan {
    root: std::path::PathBuf,
}

impl DriftScan {
    fn new(root: impl Into<std::path::PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn assert_absent(&self, forbidden_fragments: &[&str]) {
        let mut violations = Vec::new();
        self.collect_violations("src/lib.rs", forbidden_fragments, &mut violations);
        assert!(
            violations.is_empty(),
            "terminal prompt-gate records belong to signal-persona-terminal:\n{}",
            violations.join("\n")
        );
    }

    fn collect_violations(
        &self,
        relative_path: &str,
        forbidden_fragments: &[&str],
        violations: &mut Vec<String>,
    ) {
        let path = self.root.join(relative_path);
        let content = std::fs::read_to_string(&path).expect("scan source file");
        for fragment in forbidden_fragments {
            if content.contains(fragment) {
                violations.push(format!("{relative_path} contains {fragment}"));
            }
        }
    }
}
