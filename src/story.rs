use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, sync_channel, SyncSender};
use std::thread;

use crate::ArcYard;

pub trait Teller {
	type V: Send + Clone;
	type A: Send;

	fn create() -> Self::V;

	fn update(vision: &Self::V, action: Self::A) -> AfterUpdate<Self::V>;

	fn yard(_vision: &Self::V, _link: &Link<Self::A>) -> Option<ArcYard> { None }

	fn begin_story() -> Story<Self> where Self: std::marker::Sized + 'static {
		let (msg_sender, msg_receiver) = sync_channel::<Msg<Self>>(100);
		thread::spawn(move || {
			let mut vision = Self::create();
			let mut vision_senders: HashMap<i32, SyncSender<Self::V>> = HashMap::new();
			loop {
				let msg = msg_receiver.recv().unwrap();
				match msg {
					Msg::Subscribe(subscriber_id, vision_sender) => {
						assert!(!vision_senders.contains_key(&subscriber_id));
						vision_sender.send(vision.clone()).unwrap();
						vision_senders.insert(subscriber_id, vision_sender);
					}
					Msg::Update(action) =>
						match Self::update(&vision, action) {
							AfterUpdate::Ignore => (),
							AfterUpdate::Revise(next) => {
								vision = next;
								vision_senders.iter().for_each(|(_, it)| {
									it.send(vision.clone()).unwrap()
								});
							}
						},
				}
			}
		});
		Story { sender: msg_sender }
	}
}

enum Msg<T: Teller> {
	Subscribe(i32, SyncSender<T::V>),
	Update(T::A),
}

#[derive(Clone, Debug)]
pub struct Story<T: Teller> {
	sender: SyncSender<Msg<T>>
}

impl<T: Teller + 'static> Story<T> {
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
}

#[derive(Clone)]
pub struct Link<A> {
	tx: Arc<dyn Fn(A) + Send + Sync>,
}

impl<A: Send> Link<A> {
	pub fn callback<Ctx>(&self, into_action: impl Fn(Ctx) -> A + Send) -> impl Fn(Ctx) {
		let tx = self.tx.to_owned();
		move |ctx: Ctx| {
			let action = into_action(ctx);
			(*tx)(action);
		}
	}
}
