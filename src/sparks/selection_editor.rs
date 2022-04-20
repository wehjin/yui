use std::fmt;

use selection_editor::Action;

use crate::{AfterFlow, ArcYard, Before, Cling, Confine, Create, FillColor, FillGrade, Flow, Link, Pack, Padding, selection_editor, SenderLink, Spark, SyncLink};
use crate::app::Edge;
use crate::palette::StrokeColor;
use crate::selection_editor::SelectionEditor;
use crate::yard::{ButtonState, Pressable};
use crate::yui::prelude::yard;

pub struct SelectionEditorSpark<T> {
	pub selected: usize,
	pub choices: Vec<T>,
}

impl<T: Clone + Send + fmt::Display> Spark for SelectionEditorSpark<T> {
	type State = SelectionEditor<T>;
	type Action = Action;
	type Report = Option<(usize, T)>;

	fn create<E: Edge>(&self, _ctx: &Create<Self::Action, Self::Report, E>) -> Self::State {
		SelectionEditor::new(self.selected, &self.choices)
	}

	fn flow(&self, action: Self::Action, ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		let editor = ctx.state().clone().into_next(action);
		if editor.is_closed {
			AfterFlow::Close(Some(editor.selection.to_owned()))
		} else {
			AfterFlow::Revise(editor)
		}
	}

	fn render(state: &Self::State, link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		let content = if state.choices.is_empty() {
			yard::label("Empty", StrokeColor::CommentOnBackground, Cling::Center)
		} else {
			let yards = state.choices.iter().enumerate().map(|(i, it)| {
				let (text, color) = if state.selected_index() == i {
					(format!("{}", it).to_uppercase(), StrokeColor::BodyOnBackground)
				} else {
					(format!("{}", it), StrokeColor::EnabledOnBackground)
				};
				let label = yard::label(text, color, Cling::Center);
				let yard = label.pressable(link.map(move |_| Action::SelectIndex(i)));
				yard
			}).collect::<Vec<_>>();

			let link = link.clone();
			let send_action = SyncLink::wrap_sink(move |action| {
				link.send(Action::ToListArt(action));
			});
			yard::list(yards, state.list_art.clone(), send_action)
		};
		let close = yard::button("Close", ButtonState::enabled(link.map(|_| Action::Close))).confine_width(9, Cling::Center);
		let yard = content.pack_bottom(1, close).pad(1)
			.before(yard::fill(FillColor::Background, FillGrade::Plain));
		Some(yard)
	}
}

