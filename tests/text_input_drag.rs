use floem::event::Event;
use floem::headless::{HeadlessHarness, TestRoot};
use floem::reactive::{RwSignal, SignalGet};
use floem::ui_events::keyboard::{Code, Key, KeyState, KeyboardEvent, Modifiers, NamedKey};
use floem::ui_events::pointer::{
    PointerButton, PointerButtonEvent, PointerEvent, PointerId, PointerInfo, PointerState,
    PointerType,
};
use floem::views::{Decorators, TextInput};
use floem::{HasViewId, ViewId};

const TEXT: &str = "the quick brown fox";
/// Left of the text's first letter and right of its last, on its line.
const FRONT: f64 = 1.0;
const BACK: f64 = 290.0;
const Y: f64 = 10.0;

/// An input over `TEXT` with the keyboard, filling the window's width.
fn input() -> (HeadlessHarness, RwSignal<String>, ViewId) {
    let root = TestRoot::new();
    let buffer = RwSignal::new(TEXT.to_string());
    let view = TextInput::new(buffer).style(|s| s.width(300.0));
    let id = view.view_id();
    let mut harness = HeadlessHarness::new_with_size(root, view, 300.0, 100.0);
    harness.rebuild();
    harness.paint();
    id.request_focus();
    harness.rebuild();
    assert!(harness.is_focused(id), "the input takes the keyboard");
    (harness, buffer, id)
}

/// The primary button going down or up at `x`.
fn button(x: f64, down: bool) -> Event {
    let mut state = PointerState::default();
    state.position.x = x;
    state.position.y = Y;
    state.count = 1;
    let event = PointerButtonEvent {
        state,
        button: Some(PointerButton::Primary),
        pointer: PointerInfo {
            pointer_id: Some(PointerId::PRIMARY),
            persistent_device_id: None,
            pointer_type: PointerType::Mouse,
        },
    };
    Event::Pointer(if down {
        PointerEvent::Down(event)
    } else {
        PointerEvent::Up(event)
    })
}

fn key(key: Key, code: Code) -> Event {
    Event::Key(KeyboardEvent {
        state: KeyState::Down,
        key,
        code,
        location: Default::default(),
        modifiers: Modifiers::empty(),
        repeat: false,
        is_composing: false,
    })
}

fn typed(harness: &mut HeadlessHarness, ch: &str, code: Code) {
    harness.dispatch_event(key(Key::Character(ch.into()), code));
    harness.rebuild();
}

/// A drag selects, and what is typed next takes the selection's place.
#[test]
fn a_drag_selects_what_the_next_letter_replaces() {
    let (mut harness, buffer, _) = input();
    harness.dispatch_event(button(FRONT, true));
    harness.pointer_move(BACK, Y);
    harness.dispatch_event(button(BACK, false));

    typed(&mut harness, "x", Code::KeyX);
    assert_eq!(buffer.get(), "x");
}

/// A trackpad's tap-drag and three-finger drag hold the button for a
/// moment after the fingers lift, so a letter can come before the
/// release. The pointer's next move used to select again, from the cursor
/// the letter left, and the letter after that replaced the first.
#[test]
fn a_key_that_edits_ends_the_drag() {
    let (mut harness, buffer, _) = input();
    harness.dispatch_event(button(FRONT, true));
    harness.pointer_move(BACK, Y);

    typed(&mut harness, "x", Code::KeyX);
    assert_eq!(buffer.get(), "x", "the letter takes the selection");

    harness.pointer_move(FRONT, Y);
    typed(&mut harness, "y", Code::KeyY);
    assert_eq!(buffer.get(), "xy", "and the pointer selects no more");

    harness.dispatch_event(button(FRONT, false));
    typed(&mut harness, "z", Code::KeyZ);
    assert_eq!(buffer.get(), "xyz");
}

/// An arrow that only lets a selection go, the cursor already at its end,
/// ends the drag too.
#[test]
fn a_key_that_moves_the_cursor_ends_the_drag() {
    let (mut harness, buffer, _) = input();
    harness.dispatch_event(button(FRONT, true));
    harness.pointer_move(BACK, Y);

    harness.dispatch_event(key(Key::Named(NamedKey::ArrowRight), Code::ArrowRight));
    harness.rebuild();
    harness.pointer_move(FRONT, Y);
    typed(&mut harness, "!", Code::Digit1);
    assert_eq!(buffer.get(), "the quick brown fox!");
}

/// A modifier on its own changes nothing, and the drag goes on.
#[test]
fn a_modifier_alone_leaves_the_drag_going() {
    let (mut harness, buffer, _) = input();
    harness.dispatch_event(button(BACK, true));
    harness.dispatch_event(Event::Key(KeyboardEvent {
        state: KeyState::Down,
        key: Key::Named(NamedKey::Shift),
        code: Code::ShiftLeft,
        location: Default::default(),
        modifiers: Modifiers::SHIFT,
        repeat: false,
        is_composing: false,
    }));
    harness.pointer_move(FRONT, Y);

    typed(&mut harness, "x", Code::KeyX);
    assert_eq!(buffer.get(), "x", "the drag selected to the front");
}

#[test]
fn the_press_after_a_key_ended_a_drag_drags_again() {
    let (mut harness, buffer, _) = input();
    harness.dispatch_event(button(FRONT, true));
    harness.pointer_move(BACK, Y);
    typed(&mut harness, "x", Code::KeyX);
    harness.dispatch_event(button(BACK, false));

    harness.dispatch_event(button(FRONT, true));
    harness.pointer_move(BACK, Y);
    harness.dispatch_event(button(BACK, false));
    typed(&mut harness, "y", Code::KeyY);
    assert_eq!(buffer.get(), "y");
}
