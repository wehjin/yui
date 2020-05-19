use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread;

use crate::{ArcYard, Link, Story};
use crate::app::Edge;
use crate::story::scope::StoryScope;

pub trait Wheel: 'static {
	type State: Send + Clone;
	type Action: Send;
	type Report: Send;

	fn build(report_link: Option<Link<Self::Report>>) -> Self::State;
	fn roll(ctx: &impl RollContext<Self::State, Self::Action>, action: Self::Action) -> AfterRoll<Self::State>;
	fn yard(_state: &Self::State, _link: &Link<Self::Action>) -> Option<ArcYard> { None }
	fn launch(edge: Option<Edge>, report_link: Option<Link<Self::Report>>) -> Story<Self> where Self: std::marker::Sized + 'static {
		let (tx, rx) = sync_channel::<Msg<Self>>(100);
		let story = Story { tx };
		let action_link = story.link().clone();
		thread::spawn(move || {
			let mut ctx = StoryScope::new(
				Self::build(report_link),
				action_link,
				edge,
			);
			for msg in rx {
				match msg {
					Msg::Subscribe(subscriber_id, watcher) =>
						ctx.add_watcher(subscriber_id, watcher),
					Msg::Update(action) => match Self::roll(&ctx, action) {
						AfterRoll::ReviseQuietly(next) => ctx.set_vision(next, false),
						AfterRoll::Revise(next) => ctx.set_vision(next, true),
						AfterRoll::Ignore => (),
					},
				}
			}
		});
		story
	}
}

pub trait RollContext<State, Action> {
	fn state(&self) -> &State;
	fn link(&self) -> &Link<Action>;
	fn start_prequel<T: Wheel>(&self) -> Story<T>;
	fn end_prequel(&self);
}


pub enum AfterRoll<State> {
	Ignore,
	Revise(State),
	ReviseQuietly(State),
}

pub(crate) enum Msg<T: Wheel> {
	Subscribe(i32, SyncSender<T::State>),
	Update(T::Action),
}
