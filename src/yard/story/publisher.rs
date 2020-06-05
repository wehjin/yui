use std::error::Error;
use std::sync::mpsc::{Receiver, sync_channel};
use std::thread;

use crate::{ArcYard, Spark, Story};
use crate::yard::YardPublisher;

pub(crate) struct StoryYardPublisher<S: Spark> {
	pub story: Story<S>
}

impl<S> YardPublisher for StoryYardPublisher<S> where S: Spark + Sync + Send + 'static {
	fn subscribe(&self) -> Result<Receiver<ArcYard>, Box<dyn Error>> {
		let visions = self.story.visions(rand::random())?;
		let (tx_yard, rx_yard) = sync_channel::<ArcYard>(64);
		let link = self.story.link();
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
				if let Some(yard) = S::yard(&vision, &link) {
					if let Err(_) = tx_yard.send(yard) {
						done = true;
					}
				};
			}
		});
		Ok(rx_yard)
	}
}
