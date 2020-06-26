use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread;

use crate::{ArcYard, Link, Story};
use crate::app::Edge;
use crate::story::scope::StoryScope;

pub trait Spark {
	type State: Send + Clone;
	type Action: Send;
	type Report: Send;

	fn spark(self, edge: Option<Edge>, report_link: Option<Link<Self::Report>>) -> Story<Self>
		where Self: Sized + Sync + Send + 'static
	{
		let (tx, rx) = sync_channel::<Msg<Self>>(100);
		let story = Story { tx };
		let action_link = story.link().clone();
		thread::spawn(move || {
			let state = self.create(&Create {
				action_link: action_link.clone(),
				edge: edge.clone(),
				report_link: report_link.clone(),
			});
			let mut ctx = StoryScope::new(state, action_link, edge, move |report| {
				match &report_link {
					None => {}
					Some(link) => link.send(report),
				}
			});
			for msg in rx {
				match msg {
					Msg::Subscribe(subscriber_id, watcher) => ctx.add_watcher(subscriber_id, watcher),
					Msg::Update(action) => match Self::flow(&self, &ctx, action) {
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

	fn render(_state: &Self::State, _link: &Link<Self::Action>) -> Option<ArcYard> { None }
	fn flow(&self, flow: &impl Flow<Self::State, Self::Action, Self::Report>, action: Self::Action) -> AfterFlow<Self::State, Self::Report>;
	fn create(&self, create: &Create<Self::Action, Self::Report>) -> Self::State;
}

pub struct Create<Action, Report> {
	action_link: Link<Action>,
	report_link: Option<Link<Report>>,
	edge: Option<Edge>,
}

impl<Action, Report> Create<Action, Report> {
	pub fn link(&self) -> &Link<Action> { &self.action_link }
	pub fn report_link(&self) -> &Option<Link<Report>> { &self.report_link }
	pub fn edge(&self) -> &Option<Edge> { &self.edge }
}

pub trait Flow<State, Action, Report> {
	fn state(&self) -> &State;
	fn link(&self) -> &Link<Action>;
	fn start_prequel<S>(&self, spark: S, on_report: impl Fn(S::Report) + Sync + Send + 'static) -> Story<S> where S: Spark + Sync + Send + 'static;
	fn end_prequel(&self);
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
