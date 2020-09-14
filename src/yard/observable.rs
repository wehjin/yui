use std::error::Error;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, sync_channel, SyncSender};

use crate::ArcYard;

pub trait YardPublisher {
	fn subscribe(&self) -> Result<Receiver<ArcYard>, Box<dyn Error>>;
}

impl YardPublisher for Arc<dyn YardPublisher> {
	fn subscribe(&self) -> Result<Receiver<ArcYard>, Box<dyn Error>> { self.deref().subscribe() }
}

pub enum YardControlMsg {
	On(SyncSender<ArcYard>),
	Off,
	Forward(ArcYard),
}

impl YardPublisher for SyncSender<YardControlMsg> {
	fn subscribe(&self) -> Result<Receiver<ArcYard>, Box<dyn Error>> {
		let (tx, rx) = sync_channel(100);
		self.send(YardControlMsg::On(tx)).unwrap();
		Ok(rx)
	}
}
