use crate::selection_editor::{Action, IndexState, SelectionEditor};

#[test]
fn editor_works() {
	let start = SelectionEditor::new(0, &[1u8, 2]);
	assert_states(&start, &[IndexState::SelectedWithFocus, IndexState::Unselected]);

	let underflow = start.into_next(Action::FocusBackward);
	assert_states(&underflow, &[IndexState::SelectedWithFocus, IndexState::Unselected]);

	let forward = underflow.into_next(Action::FocusForward);
	assert_states(&forward, &[IndexState::Selected, IndexState::UnselectedWithFocus]);

	let press = forward.into_next(Action::Press);
	assert_states(&press, &[IndexState::Selected, IndexState::PressedUnselected]);
	assert!(press.selection.is_none());

	let select = press.into_next(Action::Select);
	assert_states(&select, &[IndexState::Unselected, IndexState::SelectedWithFocus]);
	assert!(select.selection.is_some());

	let overflow = select.into_next(Action::FocusForward);
	assert_states(&overflow, &[IndexState::Unselected, IndexState::SelectedWithFocus]);

	let backward = overflow.into_next(Action::FocusBackward);
	assert_states(&backward, &[IndexState::UnselectedWithFocus, IndexState::Selected]);

	let close = backward.into_next(Action::Close);
	assert_states(&close, &[IndexState::UnselectedWithFocus, IndexState::Selected]);
	assert!(close.selection.is_some());
}

fn assert_states(editor: &SelectionEditor<u8>, expected_states: &[IndexState]) {
	let actual_states = (0..editor.choices.len()).map(|it| editor.index_state(it)).collect::<Vec<_>>();
	assert_eq!(&actual_states, &expected_states);
}