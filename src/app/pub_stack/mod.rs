use std::sync::mpsc::SyncSender;

use crate::{AfterFlow, ArcYard, Create, Fade, Flow, SenderLink, story, yard};
use crate::app::Edge;
use crate::yard::YardControlMsg;

pub mod story_stack {
	use rand::random;

	use crate::{AfterFlow, ArcYard, Create, FillColor, FillGrade, Flow, SenderLink, Spark, StoryId, yard};
	use crate::app::Edge;

	#[derive(Debug, Clone)]
	pub enum StoryStackAction {
		PushStory(StoryId),
		PopStory(StoryId),
	}

	#[derive(Debug, Clone)]
	pub struct StoryStackModel {
		story_ids: Vec<StoryId>,
		yard_ids: Vec<i32>,
	}

	impl StoryStackModel {
		pub fn push_story(&mut self, story_id: StoryId) {
			if !self.story_ids.contains(&story_id) {
				self.story_ids.push(story_id);
				self.yard_ids.push(random())
			}
		}
		pub fn pop_story(&mut self, story_id: StoryId) {
			if let Some(index) = self.story_position(story_id) {
				self.story_ids.truncate(index);
				self.yard_ids.truncate(index);
			}
		}
		pub fn story_position(&self, story_id: StoryId) -> Option<usize> {
			self.story_ids.iter().position(|existing| *existing == story_id)
		}
	}

	pub struct StoryStack {}

	impl Spark for StoryStack {
		type State = StoryStackModel;
		type Action = StoryStackAction;
		type Report = ();

		fn create<E: Edge + Clone + Send + 'static>(&self, _ctx: &Create<Self::Action, Self::Report, E>) -> Self::State {
			StoryStackModel { story_ids: Vec::new(), yard_ids: Vec::new() }
		}

		fn flow(&self, action: Self::Action, ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
			match action {
				StoryStackAction::PushStory(story_id) => {
					let mut model = ctx.state().clone();
					model.push_story(story_id);
					AfterFlow::Revise(model)
				}
				StoryStackAction::PopStory(story_id) => {
					let mut model = ctx.state().clone();
					model.pop_story(story_id);
					if model.story_ids.is_empty() {
						AfterFlow::Close(None)
					} else {
						AfterFlow::Revise(model)
					}
				}
			}
		}

		fn render(state: &Self::State, _action_link: &SenderLink<Self::Action>) -> Option<ArcYard> {
			let model = state;
			let rear = yard::fill(FillColor::Background, FillGrade::Plain);
			let yard_stack = (0..model.story_ids.len()).fold(rear, |rear, index| {
				let story_id = model.story_ids[index];
				let yard_id = model.yard_ids[index];
				let story_yard = yard::story(yard_id, story_id);
				let indent = index as i32 * 2;
				yard::fade((indent, indent), rear, story_yard)
			});
			Some(yard_stack)
		}
	}
}

pub(crate) struct PubStack {}

impl story::Spark for PubStack {
	type State = Vec<SyncSender<YardControlMsg>>;
	type Action = Action;
	type Report = ();

	fn create<E: Edge>(&self, _create: &Create<Self::Action, Self::Report, E>) -> Self::State { Vec::new() }

	fn flow(&self, action: Self::Action, flow: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		match action {
			Action::Pop => {
				if flow.state().len() == 1 {
					flow.report(());
					AfterFlow::Ignore
				} else {
					let mut next_state = flow.state().to_vec();
					next_state.pop();
					AfterFlow::Revise(next_state)
				}
			}
			Action::Push(front) => {
				let mut next_state = flow.state().to_vec();
				next_state.push(front);
				AfterFlow::Revise(next_state)
			}
			Action::Refresh => {
				flow.redraw();
				AfterFlow::Ignore
			}
		}
	}

	fn render(vision: &Self::State, link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		let refresh = link.clone().map(|_| Action::Refresh);
		if let Some(first_publisher) = vision.first() {
			let publisher = yard::publisher(first_publisher, refresh.clone());
			let yard = vision[1..].iter().fold(
				publisher,
				|rear_yard, publisher| {
					let fore_yard = yard::publisher(publisher, refresh.clone());
					rear_yard.fade((4, 4), fore_yard)
				},
			);
			Some(yard)
		} else {
			None
		}
	}
}

pub(crate) enum Action {
	Push(SyncSender<YardControlMsg>),
	Pop,
	Refresh,
}
