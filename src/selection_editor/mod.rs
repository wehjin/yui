#[cfg(test)]
mod tests;

/// Selects an item from a vector of items.
#[derive(Clone, Debug)]
pub struct SelectionEditor<T: Clone> {
	pub choices: Vec<T>,
	pub start_index: usize,
	pub focused_index: usize,
	pub is_pressed: bool,
	pub selection: Option<(usize, T)>,
	pub is_closed: bool,
}

/// Enumerates the actions available to a SelectionEditor.
pub enum Action {
	FocusForward,
	FocusBackward,
	Press,
	Select,
	Close,
}

/// Enumerates the states of an index.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum IndexState {
	Selected,
	Unselected,
	SelectedWithFocus,
	UnselectedWithFocus,
	PressedSelected,
	PressedUnselected,
}

impl<T: Clone> SelectionEditor<T> {
	/// Construct the editor.
	pub fn new(index: usize, choices: &[T]) -> Self {
		SelectionEditor {
			choices: choices.to_vec(),
			start_index: index,
			focused_index: index,
			is_pressed: false,
			selection: None,
			is_closed: false,
		}
	}
	/// Read the index of the currently selected item.
	pub fn selected_index(&self) -> usize {
		if let Some((index, _)) = self.selection {
			index
		} else {
			self.start_index
		}
	}
	/// Determine the state of an index.
	pub fn index_state(&self, index: usize) -> IndexState {
		let selected_index = self.selected_index();
		if self.is_pressed && self.focused_index == index {
			// Pressed
			if index == selected_index { IndexState::PressedSelected } else { IndexState::PressedUnselected }
		} else if self.focused_index == index {
			// Focused
			if index == selected_index { IndexState::SelectedWithFocus } else { IndexState::UnselectedWithFocus }
		} else {
			// Plain
			if index == selected_index { IndexState::Selected } else { IndexState::Unselected }
		}
	}
	/// Move the editor to another state.
	pub fn into_next(self, action: Action) -> Self {
		if self.is_closed { self } else {
			match action {
				Action::FocusForward => SelectionEditor {
					focused_index: (self.focused_index as i64 + 1).min(self.choices.len() as i64 - 1) as usize,
					is_pressed: false,
					..self
				},
				Action::FocusBackward => SelectionEditor {
					focused_index: (self.focused_index as i64 - 1).max(0) as usize,
					is_pressed: false,
					..self
				},
				Action::Press => SelectionEditor {
					is_pressed: true,
					..self
				},
				Action::Select => SelectionEditor {
					is_pressed: false,
					selection: if self.focused_index == self.start_index {
						None
					} else {
						let value = (self.focused_index, self.choices[self.focused_index].clone());
						Some(value)
					},
					..self
				},
				Action::Close => SelectionEditor {
					is_closed: true,
					is_pressed: false,
					..self
				}
			}
		}
	}
}

