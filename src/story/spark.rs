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
			let state = self.create(report_link.clone());
			let mut ctx = StoryScope::new(state, action_link, edge, move |report| {
				match &report_link {
					None => {}
					Some(link) => link.send(report),
				}
			});
			for msg in rx {
				match msg {
					Msg::Subscribe(subscriber_id, watcher) =>
						ctx.add_watcher(subscriber_id, watcher),
					Msg::Update(action) => match Self::flow(&ctx, action) {
						AfterFlow::ReviseQuietly(next) => ctx.set_vision(next, false),
						AfterFlow::Revise(next) => ctx.set_vision(next, true),
						AfterFlow::Ignore => (),
					},
				}
			}
		});
		story
	}

	fn yard(_state: &Self::State, _link: &Link<Self::Action>) -> Option<ArcYard> { None }
	fn flow(trace: &impl Flow<Self::State, Self::Action, Self::Report>, action: Self::Action) -> AfterFlow<Self::State>;
	fn create(&self, report_link: Option<Link<Self::Report>>) -> Self::State;
}

pub trait Flow<State, Action, Report> {
	fn state(&self) -> &State;
	fn link(&self) -> &Link<Action>;
	fn start_prequel<S>(&self, spark: S, on_report: impl Fn(S::Report) + Sync + Send + 'static) -> Story<S> where S: Spark + Sync + Send + 'static;
	fn end_prequel(&self);
	fn report(&self, report: Report);
}


pub enum AfterFlow<State> {
	Ignore,
	Revise(State),
	ReviseQuietly(State),
}

pub(crate) enum Msg<S: Spark> {
	Subscribe(i32, SyncSender<S::State>),
	Update(S::Action),
}
