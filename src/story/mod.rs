use std::error::Error;
use std::sync::mpsc::{Receiver, sync_channel, SyncSender};

pub use spark::*;

use crate::Link;

mod scope;
mod spark;

#[derive(Debug)]
pub struct Story<S: Spark> {
	tx: SyncSender<Msg<S>>
}

impl<S: Spark> Clone for Story<S> {
	fn clone(&self) -> Self {
		Story { tx: self.tx.clone() }
	}
}

impl<S> Story<S>
	where S: Spark + Sync + Send + 'static
{
	pub fn link(&self) -> Link<S::Action> {
		let sender = self.tx.to_owned();
		Link::new(move |action: S::Action| { sender.send(Msg::Update(action)).unwrap(); })
	}

	pub fn visions(&self, id: i32) -> Result<Receiver<S::State>, Box<dyn Error>> {
		let (tx, rx) = sync_channel::<S::State>(100);
		let msg = Msg::Subscribe(id, tx);
		self.tx.send(msg)
			.map(|_| rx)
			.map_err(|e| e.into())
	}
}

