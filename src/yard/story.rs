use std::error::Error;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, sync_channel};
use std::thread;

use crate::{ArcYard, Story, Spark};
use crate::yard::{YardObservable, YardObservableSource};

impl<S> YardObservableSource for Story<S>
	where S: Spark + Sync + Send + 'static
{
	fn yards(&self) -> Arc<dyn YardObservable> {
		let publisher = StoryYardPublisher { story: self.clone() };
		Arc::new(publisher)
	}
}

struct StoryYardPublisher<S: Spark> {
	story: Story<S>
}

impl<S> YardObservable for StoryYardPublisher<S>
	where S: Spark + Sync + Send + 'static
{
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
