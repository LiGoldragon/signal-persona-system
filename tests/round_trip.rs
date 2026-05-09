//! Architectural-truth round-trip tests for the
//! `signal-persona-system` channel.
//!
//! Per `~/primary/skills/architectural-truth-tests.md`,
//! each variant of both enums has a witness test that
//! proves the macro-emitted type round-trips through a
//! length-prefixed Frame.

use signal_core::{FrameBody, Reply, Request, SemaVerb};
use signal_persona_system::{
    FocusObservation, Frame, InputBufferObservation, InputBufferState, ObserveFocus,
    ObserveInputBuffer, SubscribeFocus, SubscribeInputBuffer, SubscriptionAccepted,
    SubscriptionKind, SystemEvent, SystemRequest, SystemTarget, TargetNotFound, UnsubscribeFocus,
    UnsubscribeInputBuffer, WindowClosed,
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

#[test]
fn subscribe_focus_round_trips() {
    let request = SystemRequest::SubscribeFocus(SubscribeFocus { target: TARGET });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn unsubscribe_focus_round_trips() {
    let request = SystemRequest::UnsubscribeFocus(UnsubscribeFocus { target: TARGET });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn observe_focus_round_trips() {
    let request = SystemRequest::ObserveFocus(ObserveFocus { target: TARGET });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn subscribe_input_buffer_round_trips() {
    let request = SystemRequest::SubscribeInputBuffer(SubscribeInputBuffer { target: TARGET });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn unsubscribe_input_buffer_round_trips() {
    let request = SystemRequest::UnsubscribeInputBuffer(UnsubscribeInputBuffer { target: TARGET });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn observe_input_buffer_round_trips() {
    let request = SystemRequest::ObserveInputBuffer(ObserveInputBuffer { target: TARGET });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn focus_observation_round_trips_with_focused_true() {
    let event = SystemEvent::FocusObservation(FocusObservation {
        target: TARGET,
        focused: true,
        generation: 42,
    });
    let decoded = round_trip_event(event.clone());
    assert_eq!(decoded, event);
}

#[test]
fn focus_observation_round_trips_with_focused_false() {
    let event = SystemEvent::FocusObservation(FocusObservation {
        target: TARGET,
        focused: false,
        generation: 43,
    });
    let decoded = round_trip_event(event.clone());
    assert_eq!(decoded, event);
}

#[test]
fn input_buffer_observation_round_trips_for_each_state() {
    for state in [
        InputBufferState::Empty,
        InputBufferState::Occupied,
        InputBufferState::Unknown,
    ] {
        let event = SystemEvent::InputBufferObservation(InputBufferObservation {
            target: TARGET,
            state: state.clone(),
            generation: 99,
        });
        let decoded = round_trip_event(event.clone());
        assert_eq!(decoded, event);
    }
}

#[test]
fn window_closed_round_trips() {
    let event = SystemEvent::WindowClosed(WindowClosed { target: TARGET });
    let decoded = round_trip_event(event.clone());
    assert_eq!(decoded, event);
}

#[test]
fn subscription_accepted_round_trips_for_each_kind() {
    for kind in [SubscriptionKind::Focus, SubscriptionKind::InputBuffer] {
        let event = SystemEvent::SubscriptionAccepted(SubscriptionAccepted {
            target: TARGET,
            kind,
        });
        let decoded = round_trip_event(event.clone());
        assert_eq!(decoded, event);
    }
}

#[test]
fn target_not_found_round_trips() {
    let event = SystemEvent::TargetNotFound(TargetNotFound { target: TARGET });
    let decoded = round_trip_event(event.clone());
    assert_eq!(decoded, event);
}

#[test]
fn from_impl_lifts_subscribe_focus_into_request() {
    let payload = SubscribeFocus { target: TARGET };
    let request: SystemRequest = payload.clone().into();
    assert_eq!(request, SystemRequest::SubscribeFocus(payload));
}

#[test]
fn from_impl_lifts_focus_observation_into_event() {
    let payload = FocusObservation {
        target: TARGET,
        focused: true,
        generation: 1,
    };
    let event: SystemEvent = payload.into();
    assert_eq!(event, SystemEvent::FocusObservation(payload));
}
