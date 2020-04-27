use std::error::Error;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, sync_channel, SyncSender};
use std::thread;

use crate::app::AppContext;
use crate::story::scope::StoryScope;
use crate::yard::ArcYard;

mod scope;

pub trait Teller: 'static {
	type V: Send + Clone;
	type A: Send;

	fn create() -> Self::V;

	fn update(ctx: &impl UpdateContext<Self::V, Self::A>, action: Self::A) -> AfterUpdate<Self::V>;

	fn yard(_vision: &Self::V, _link: &Link<Self::A>) -> Option<ArcYard> { None }

	fn begin_story(ctx: Option<AppContext>) -> Story<Self> where Self: std::marker::Sized + 'static {
		let (msg_sender, msg_receiver) = sync_channel::<Msg<Self>>(100);
		let story = Story { sender: msg_sender };
		let link = story.link();
		thread::spawn(move || {
			let mut ctx = StoryScope::new(Self::create(), link, ctx);
			for msg in msg_receiver {
				match msg {
					Msg::Subscribe(subscriber_id, watcher) => {
						ctx.add_watcher(subscriber_id, watcher)
					}
					Msg::Update(action) => {
						match Self::update(&ctx, action) {
							AfterUpdate::ReviseQuietly(next) => ctx.set_vision(next, false),
							AfterUpdate::Revise(next) => ctx.set_vision(next, true),
							AfterUpdate::Ignore => (),
						}
					}
				}
			}
		});
		story
	}
}

pub trait UpdateContext<V, A> {
	fn vision(&self) -> &V;
	fn link(&self) -> &Link<A>;
	fn start_prequel<T: Teller>(&self) -> Story<T>;
	fn end_prequel(&self);
}


enum Msg<T: Teller> {
	Subscribe(i32, SyncSender<T::V>),
	Update(T::A),
}

#[derive(Debug)]
pub struct Story<T: Teller> {
	sender: SyncSender<Msg<T>>
}

impl<T: Teller> Clone for Story<T> {
	fn clone(&self) -> Self {
		Story { sender: self.sender.clone() }
	}
}

impl<T: Teller> Story<T> {
	pub fn link(&self) -> Link<T::A> {
		let sender = self.sender.to_owned();
		let tx = Arc::new(move |action: T::A| {
			sender.send(Msg::Update(action)).unwrap();
		});
		Link { tx }
	}

	pub fn visions(&self, id: i32) -> Result<Receiver<T::V>, Box<dyn Error>> {
		let (tx, rx) = sync_channel::<T::V>(100);
		let msg = Msg::Subscribe(id, tx);
		self.sender.send(msg)
			.map(|_| rx)
			.map_err(|e| e.into())
	}
}

pub enum AfterUpdate<Vision> {
	Ignore,
	Revise(Vision),
	ReviseQuietly(Vision),
}

pub struct Link<A> {
	tx: Arc<dyn Fn(A) + Send + Sync>,
}

impl<A> Clone for Link<A> {
	fn clone(&self) -> Self {
		Link { tx: self.tx.clone() }
	}
}

impl<A: Send> Link<A> {
	pub fn callback<Ctx>(&self, into_action: impl Fn(Ctx) -> A + Send) -> impl Fn(Ctx) {
		let tx = self.tx.to_owned();
		move |ctx: Ctx| {
			let action = into_action(ctx);
			(*tx)(action);
		}
	}
	pub fn send(&self, action: A) {
		self.callback(|a| a)(action);
	}
}
