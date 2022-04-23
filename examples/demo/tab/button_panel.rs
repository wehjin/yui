use std::collections::HashMap;

use rand::random;

use yui::{Sendable, SenderLink};

use yui::palette::FillGrade::Plain;
use yui::prelude::*;
use yui::sparks::selection_editor::SelectionEditorSpark;
use yui::yard::{ButtonAction, ButtonModel, SubmitAffordance};

use crate::AppTab;

#[derive(Debug)]
pub struct ButtonDemo {}

#[derive(Copy, Clone)]
pub enum Choice { Beavis, Hall }

impl Choice {
	pub fn button_names(&self) -> (&str, &str) {
		match self {
			Choice::Beavis => (&"Beavis", &"Butthead"),
			Choice::Hall => (&"Hall", &"Oates"),
		}
	}
}

#[derive(Clone)]
pub struct State {
	pub choice: Choice,
	pub buttons: HashMap<(usize, usize), ButtonModel>,
}

impl State {
	pub fn press_button(&self, key: (usize, usize)) -> Self {
		let mut buttons = self.buttons.clone();
		let button = buttons.remove(&key).expect("proper button");
		let pressed_button = button.update(ButtonAction::Press);
		buttons.insert(key, pressed_button);
		State { choice: self.choice, buttons }
	}
	pub fn release_buttons(&self) -> Self {
		let buttons = self.buttons.iter()
			.map(|(key, button)| {
				let released_button = button.update(ButtonAction::Release);
				(*key, released_button)
			})
			.collect();
		State { choice: self.choice, buttons }
	}
	pub fn choose(&self, choice: Choice) -> Self {
		let button_names = choice.button_names();
		let buttons = self.buttons.iter().map(|((left_right, top_bottom), button)| {
			let button = if *left_right == 1 {
				let button_name = if *top_bottom == 0 { button_names.0 } else { button_names.1 };
				button.set_label(button_name)
			} else {
				button.clone()
			};
			((*left_right, *top_bottom), button)
		}).collect();
		State { choice, buttons }
	}
}

#[derive(Clone)]
pub enum Action {
	PressButton(usize, usize),
	OfferChoice,
	Choose(Option<usize>),
	ReleaseIgnore,
}

impl Sendable for Action {}

impl Spark for ButtonDemo {
	type State = State;
	type Action = Action;
	type Report = usize;

	fn create(&self, ctx: &Create<Self::Action, Self::Report>) -> Self::State {
		let choice = Choice::Beavis;
		let button_names = choice.button_names();
		let mut buttons = HashMap::new();
		buttons.insert((0, 0), ButtonModel {
			id: random(),
			label: "Garfunkel".to_string(),
			release_trigger: Action::ReleaseIgnore.to_send(&ctx.link()),
			affordance: SubmitAffordance::enabled(Action::PressButton(0, 0).to_sync(ctx.link())),
		});
		buttons.insert((0, 1), ButtonModel {
			id: random(),
			label: "Simon".to_string(),
			release_trigger: Action::ReleaseIgnore.to_send(&ctx.link()),
			affordance: SubmitAffordance::enabled(Action::PressButton(0, 1).to_sync(ctx.link())),
		});
		buttons.insert((1, 0), ButtonModel {
			id: random(),
			label: button_names.0.to_string(),
			release_trigger: Action::OfferChoice.to_send(&ctx.link()),
			affordance: SubmitAffordance::enabled(Action::PressButton(1, 0).to_sync(ctx.link())),
		});
		buttons.insert((1, 1), ButtonModel {
			id: random(),
			label: button_names.1.to_string(),
			release_trigger: Action::ReleaseIgnore.to_send(&ctx.link()),
			affordance: SubmitAffordance::enabled(Action::PressButton(1, 1).to_sync(ctx.link())),
		});
		State { choice, buttons }
	}

	fn flow(&self, action: Self::Action, ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		let state = ctx.state();
		match action {
			Action::PressButton(left_right, top_bottom) => {
				let state = state.press_button((left_right, top_bottom));
				AfterFlow::Revise(state)
			}
			Action::ReleaseIgnore => {
				let state = state.release_buttons();
				AfterFlow::Revise(state)
			}
			Action::OfferChoice => {
				let choices = vec!["Beavis", "Hall  "];
				let selected = match state.choice {
					Choice::Beavis => 0,
					Choice::Hall => 1,
				};
				let choose_spark = SelectionEditorSpark { selected, choices };
				let choose_link = ctx.link()
					.map(|choice: Option<(usize, &'static str)>| {
						let choice = choice.map(|(index, _)| index);
						Action::Choose(choice)
					});
				ctx.start_prequel(choose_spark, choose_link);
				let state = ctx.state().release_buttons();
				AfterFlow::Revise(state)
			}
			Action::Choose(choice) => {
				if let Some(choice) = choice {
					let choice = if choice == 0 { Choice::Beavis } else { Choice::Hall };
					let state = state.choose(choice);
					AfterFlow::Revise(state)
				} else {
					AfterFlow::Ignore
				}
			}
		}
	}

	fn render(state: &Self::State, _link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		fn ordered_buttons(state: &State, col: usize) -> Vec<ArcYard> {
			let mut buttons = state.buttons.iter().filter_map(|((left_right, top_bottom), value)| {
				if *left_right == col { Some((*top_bottom, yard::button(value))) } else { None }
			}).collect::<Vec<_>>();
			buttons.sort_by_key(|it| it.0);
			buttons.into_iter().map(|(_, button)| button).collect()
		}
		let dark_half = yard::trellis(1, 1, Cling::Center, ordered_buttons(state, 1))
			.pad(3)
			.before(yard::fill(FillColor::Primary, Plain));
		let light_half = yard::trellis(1, 1, Cling::Center, ordered_buttons(state, 0))
			.pad(3)
			.before(yard::fill(FillColor::Background, Plain));
		let content = light_half.pack_right(40, dark_half);
		let page = AppTab::Buttons.page(content);
		Some(page)
	}
}