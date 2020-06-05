use std::error::Error;
use std::sync::mpsc::{Receiver, sync_channel, SyncSender};

pub use spark::*;

use crate::Link;

mod scope;
mod spark;

#[derive(Debug)]
pub struct Story<Spk: Spark> { tx: SyncSender<Msg<Spk>> }

impl<Spk: Spark> Clone for Story<Spk> {
	fn clone(&self) -> Self {
		Story { tx: self.tx.clone() }
	}
}

impl<Spk> Story<Spk> where Spk: Spark + Sync + Send + 'static
{
	pub fn link(&self) -> Link<Spk::Action> {
		let sender = self.tx.to_owned();
		Link::new(move |action: Spk::Action| { sender.send(Msg::Update(action)).unwrap(); })
	}

	pub fn visions(&self, id: i32) -> Result<Receiver<Spk::State>, Box<dyn Error>> {
		let (tx, rx) = sync_channel::<Spk::State>(100);
		let msg = Msg::Subscribe(id, tx);
		self.tx.send(msg)
			.map(|_| rx)
			.map_err(|e| e.into())
	}
}

