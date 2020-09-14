use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread;

use crate::{ArcYard, SenderLink, Story};
use crate::app::Edge;
use crate::story::scope::StoryScope;

pub fn spark<S: Spark>(spark: S, edge: Option<Edge>, report_link: Option<SenderLink<S::Report>>) -> Story<S>
	where S: Sized + Sync + Send + 'static
{
	let (tx, rx) = sync_channel::<Msg<S>>(100);
	let story = Story { tx };
	let action_link = story.link().clone();
	thread::spawn(move || {
		let state = spark.create(&Create {
			action_link: action_link.clone(),
			edge: edge.clone(),
			report_link: report_link.clone(),
		});
		let on_report = match &report_link {
			None => SenderLink::ignore(),
			Some(link) => link.clone(),
		};
		let mut ctx = StoryScope::new(state, action_link, edge, on_report);
		for msg in rx {
			match msg {
				Msg::Subscribe(subscriber_id, watcher) => ctx.add_watcher(subscriber_id, watcher),
				Msg::Update(action) => match S::flow(&spark, action, &ctx) {
					AfterFlow::ReviseQuietly(next) => ctx.set_vision(next, false),
					AfterFlow::Revise(next) => ctx.set_vision(next, true),
					AfterFlow::Ignore => (),
					AfterFlow::Report(report) => ctx.report(report),
					AfterFlow::Close(report) => {
						if let Some(report) = report { ctx.report(report) }
						ctx.end_prequel();
					}
				},
			}
		}
	});
	story
}

pub trait Spark {
	//! Sparks specify the state of a story, the actions that change it, and
	//! the reports it emits.

	/// Specifies state that evolves with the story.
	type State: Send + Clone;
	/// Specifies actions that move the story forward.
	type Action: Send;
	/// Specifies reports the story emit.
	type Report: Send;

	/// Produce the starting state for a story generated by this spark.
	fn create(&self, ctx: &Create<Self::Action, Self::Report>) -> Self::State;

	/// Produce a new state after an action moves the story forward.
	fn flow(&self, action: Self::Action, ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report>;

	/// Produce a rendering for a state of the story.
	fn render(_state: &Self::State, _link: &SenderLink<Self::Action>) -> Option<ArcYard> { None }
}

pub struct Create<Action, Report> {
	action_link: SenderLink<Action>,
	report_link: Option<SenderLink<Report>>,
	edge: Option<Edge>,
}

impl<Action, Report> Create<Action, Report> {
	pub fn link(&self) -> &SenderLink<Action> { &self.action_link }
	pub fn report_link(&self) -> &Option<SenderLink<Report>> { &self.report_link }
	pub fn edge(&self) -> &Option<Edge> { &self.edge }
}

pub trait Flow<State, Action, Report> {
	//! TODO: Move start_prequel and end_prequel into edge component.
	fn state(&self) -> &State;
	fn link(&self) -> &SenderLink<Action>;
	fn start_prequel<S>(&self, spark: S, on_report: impl Fn(S::Report) + Sync + Send + 'static) -> Story<S> where S: Spark + Sync + Send + 'static;
	fn end_prequel(&self);
	fn redraw(&self);
	fn report(&self, report: Report);
}


pub enum AfterFlow<State, Report> {
	Ignore,
	Report(Report),
	Close(Option<Report>),
	Revise(State),
	ReviseQuietly(State),
}

pub(crate) enum Msg<S: Spark> {
	Subscribe(i32, SyncSender<S::State>),
	Update(S::Action),
}
