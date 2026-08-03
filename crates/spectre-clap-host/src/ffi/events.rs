// =============================================================================
// File: crates/spectre-clap-host/src/ffi/events.rs
// Layer: CLAP host
// Purpose: Empty input/output event lists for the no-events process bridge
// Status: Implemented; stateless stubs. Note/param event routing lands later.
// Notes: clap_process requires non-null in_events/out_events. Until the event
//        bridge exists, the host supplies a list that yields nothing and accepts
//        nothing. Both are stateless, so one shared 'static of each is sound.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use clap_sys::events::{clap_event_header, clap_input_events, clap_output_events};

// An input event list that is always empty
static EMPTY_INPUT: clap_input_events = clap_input_events {
    ctx: std::ptr::null_mut(),
    size: Some(input_size),
    get: Some(input_get),
};

// An output event list that accepts nothing; emitted events are dropped for now
static EMPTY_OUTPUT: clap_output_events = clap_output_events {
    ctx: std::ptr::null_mut(),
    try_push: Some(output_try_push),
};

// Shared empty input list for a process block
pub fn empty_input_events() -> *const clap_input_events {
    &EMPTY_INPUT
}

// Shared rejecting output list for a process block
pub fn empty_output_events() -> *const clap_output_events {
    &EMPTY_OUTPUT
}

// The input list holds zero events
unsafe extern "C" fn input_size(_list: *const clap_input_events) -> u32 {
    0
}

// No event is ever retrievable; never called while size() is zero
unsafe extern "C" fn input_get(
    _list: *const clap_input_events,
    _index: u32,
) -> *const clap_event_header {
    std::ptr::null()
}

// Every pushed output event is refused
unsafe extern "C" fn output_try_push(
    _list: *const clap_output_events,
    _event: *const clap_event_header,
) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_list_is_empty() {
        let list = empty_input_events();
        // SAFETY: list is the live 'static empty input list with size set
        let n = unsafe { (*list).size.unwrap()(list) };
        assert_eq!(n, 0);
    }

    #[test]
    fn output_list_refuses_events() {
        let list = empty_output_events();
        // SAFETY: list is the live 'static output list; a null event is never
        // dereferenced because the stub refuses unconditionally
        let pushed = unsafe { (*list).try_push.unwrap()(list, std::ptr::null()) };
        assert!(!pushed);
    }
}
