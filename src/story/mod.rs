use std::error::Error;
use std::sync::mpsc::{Receiver, sync_channel, SyncSender};
use std::thread;

pub use spark::*;

use crate::{ArcYard, SyncLink};
use crate::yard::YardPublisher;

mod scope;
mod spark;

/// Stories are evolving elements in an interaction.  They
/// maintain state, respond to actions, and emit reports.
/// Every story begins with a Spark.
#[derive(Debug)]
pub struct Story<Spk: Spark> { tx: SyncSender<Msg<Spk>> }

impl<Spk: Spark> Clone for Story<Spk> {
	fn clone(&self) -> Self {
		Story { tx: self.tx.clone() }
	}
}

impl<Spk> Story<Spk> where Spk: Spark + Sync + Send + 'static
{
	pub fn link(&self) -> SyncLink<Spk::Action> {
		let sender = self.tx.to_owned();
		SyncLink::new(move |action: Spk::Action| { sender.send(Msg::Update(action)).unwrap(); })
	}

	pub fn visions(&self, id: i32) -> Result<Receiver<Spk::State>, Box<dyn Error>> {
		let (tx, rx) = sync_channel::<Spk::State>(100);
		let msg = Msg::Subscribe(id, tx);
		self.tx.send(msg)
			.map(|_| rx)
			.map_err(|e| e.into())
	}
}

impl<Sprk> YardPublisher for Story<Sprk> where Sprk: Spark + Sync + Send + 'static {
	fn subscribe(&self) -> Result<Receiver<ArcYard>, Box<dyn Error>> {
		let visions = self.visions(rand::random())?;
		let (tx_yard, rx_yard) = sync_channel::<ArcYard>(64);
		let link = self.link();
		thread::spawn(move || {
			let mut done = false;
			while !done {
				let vision = {
					let mut first = visions.recv().unwrap();
					loop {
						let second = visions.try_recv();
						if second.is_err() {
							break;
						} else {
							first = second.unwrap();
						}
					}
					first
				};
				if let Some(yard) = Sprk::render(&vision, &link) {
					if let Err(_) = tx_yard.send(yard) {
						done = true;
					}
				};
			}
		});
		Ok(rx_yard)
	}
}
