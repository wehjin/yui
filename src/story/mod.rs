use std::error::Error;
use std::sync::mpsc::{channel, Receiver, Sender, sync_channel, SyncSender};
use std::thread;

pub use spark::*;

use crate::{ArcYard, SenderLink};
use crate::yard::{YardControlMsg, YardPublisher};

mod scope;
mod spark;

/// Stories are evolving elements in an interaction.  They
/// maintain state, respond to actions, and emit reports.
/// Every story begins with a Spark.
#[derive(Debug)]
pub struct Story<Spk: Spark> { tx: Sender<Msg<Spk>> }

impl<Spk: Spark> Clone for Story<Spk> {
	fn clone(&self) -> Self {
		Story { tx: self.tx.clone() }
	}
}

struct Worker {
	stopper: Option<Sender<()>>,
}

impl Worker {
	fn new() -> Self { Worker { stopper: None } }

	fn stop(&mut self) {
		if let Some(ref stopper) = self.stopper {
			stopper.send(()).expect("send () to stopper");
		} else {}
		self.stopper = None;
	}

	fn start(&mut self, stopper: Sender<()>) {
		self.stop();
		self.stopper = Some(stopper);
	}
}

impl<Spk> Story<Spk> where Spk: Spark + 'static, Spk::Action: 'static
{
	pub fn link(&self) -> SenderLink<Spk::Action> {
		let sender = self.tx.to_owned();
		SenderLink::new_f(move |action: Spk::Action| { sender.send(Msg::Update(action)).expect("send Msg::Update"); })
	}

	pub fn visions(&self, id: i32) -> Result<Receiver<Spk::State>, Box<dyn Error>> {
		let (tx, rx) = channel::<Spk::State>();
		let msg = Msg::Subscribe(id, tx);
		self.tx.send(msg)
			.map(|_| rx)
			.map_err(|e| e.into())
	}

	pub fn connect(&self) -> SyncSender<YardControlMsg> {
		let (tx, rx) = sync_channel::<YardControlMsg>(500);
		thread::spawn({
			let story = self.clone();
			let edge_tx = tx.clone();
			move || {
				let mut yards_out: Option<SyncSender<ArcYard>> = None;
				let mut worker = Worker::new();
				for msg in rx {
					match msg {
						YardControlMsg::On(sender) => {
							worker.stop();
							yards_out = Some(sender);
							let yards = story.subscribe().expect("subscribe story");
							let edge_tx = edge_tx.clone();
							let (end_tx, end_rx) = channel();
							worker.start(end_tx);
							thread::spawn(move || {
								let mut done = false;
								while !done {
									let yard = yards.recv().expect("receive yard");
									if end_rx.try_recv().is_err() {
										edge_tx.send(YardControlMsg::Forward(yard)).expect("send Forward to edge");
									} else {
										done = true
									}
								}
							});
						}
						YardControlMsg::Off => {
							worker.stop();
							yards_out = None;
						}
						YardControlMsg::Forward(yard) => {
							if let Some(ref forward) = yards_out {
								forward.send(yard).expect("send yard to sync-sender");
							}
						}
					}
				}
				worker.stop();
			}
		});
		tx
	}
}

impl<Sprk> YardPublisher for Story<Sprk> where Sprk: Spark + 'static {
	fn subscribe(&self) -> Result<Receiver<ArcYard>, Box<dyn Error>> {
		let visions = self.visions(rand::random())?;
		let (tx_yard, rx_yard) = sync_channel::<ArcYard>(64);
		thread::spawn({
			let link = self.link();
			move || {
				let mut done = false;
				while !done {
					let vision = {
						let mut first = visions.recv().expect("receive vision");
						loop {
							let second = visions.try_recv();
							if second.is_err() {
								break;
							} else {
								first = second.expect("second");
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
			}
		});
		Ok(rx_yard)
	}
}
