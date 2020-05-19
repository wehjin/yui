use std::error::Error;
use std::sync::mpsc::{Receiver, sync_channel, SyncSender};

pub use wheel::*;

use crate::Link;

mod scope;
mod wheel;

#[derive(Debug)]
pub struct Story<T: Wheel> {
	tx: SyncSender<Msg<T>>
}

impl<T: Wheel> Clone for Story<T> {
	fn clone(&self) -> Self {
		Story { tx: self.tx.clone() }
	}
}

impl<T: Wheel> Story<T> {
	pub fn link(&self) -> Link<T::Action> {
		let sender = self.tx.to_owned();
		Link::new(move |action: T::Action| { sender.send(Msg::Update(action)).unwrap(); })
	}

	pub fn visions(&self, id: i32) -> Result<Receiver<T::State>, Box<dyn Error>> {
		let (tx, rx) = sync_channel::<T::State>(100);
		let msg = Msg::Subscribe(id, tx);
		self.tx.send(msg)
			.map(|_| rx)
			.map_err(|e| e.into())
	}
}

