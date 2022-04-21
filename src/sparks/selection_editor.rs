use std::fmt;

use selection_editor::Action;

use crate::{AfterFlow, ArcYard, Before, Cling, Confine, Create, FillColor, FillGrade, Flow, Link, Pack, Padding, selection_editor, SenderLink, Spark, SyncLink};
use crate::app::Edge;
use crate::palette::StrokeColor;
use crate::selection_editor::SelectionEditor;
use crate::yard::{ButtonAction, ButtonModel, Pressable};
use crate::yui::prelude::yard;

pub struct SelectionEditorSpark<T> {
	pub selected: usize,
	pub choices: Vec<T>,
}

impl<T: Clone + Send + fmt::Display> Spark for SelectionEditorSpark<T> {
	type State = (SelectionEditor<T>, ButtonModel);
	type Action = Action;
	type Report = Option<(usize, T)>;

	fn create<E: Edge>(&self, ctx: &Create<Self::Action, Self::Report, E>) -> Self::State {
		let editor = SelectionEditor::new(self.selected, &self.choices);
		let release_trigger = ctx.link().to_trigger(Action::Close);
		let press_link = ctx.link().to_sync().map(|_| Action::UpdateButton(ButtonAction::Press));
		let button = ButtonModel::enabled("Close", release_trigger, press_link);
		(editor, button)
	}

	fn flow(&self, action: Self::Action, ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		let (editor, button) = ctx.state();
		if let Action::UpdateButton(action) = action {
			let button = button.update(action);
			AfterFlow::Revise((editor.clone(), button))
		} else {
			let editor = editor.clone().into_next(action);
			if editor.is_closed {
				ctx.link().send(Action::UpdateButton(ButtonAction::Release));
				AfterFlow::Close(Some(editor.selection.to_owned()))
			} else {
				AfterFlow::Revise((editor, button.clone()))
			}
		}
	}

	fn render(state: &Self::State, link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		let (editor, button) = state;
		let content = if editor.choices.is_empty() {
			yard::label("Empty", StrokeColor::CommentOnBackground, Cling::Center)
		} else {
			let yards = editor.choices.iter().enumerate().map(|(i, it)| {
				let (text, color) = if editor.selected_index() == i {
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
				link.send(Action::UpdateScroll(action));
			});
			yard::list(yards, editor.scroll.clone(), send_action)
		};

		let close = yard::button2(button).confine_width(9, Cling::Center);
		let yard = content.pack_bottom(1, close).pad(1)
			.before(yard::fill(FillColor::Background, FillGrade::Plain));
		Some(yard)
	}
}

