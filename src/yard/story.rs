use std::error::Error;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, sync_channel};
use std::thread;

use crate::{ArcYard, Plot, Story};
use crate::yard::{YardObservable, YardObservableSource};

impl<T: Plot> YardObservableSource for Story<T> {
	fn yards(&self) -> Arc<dyn YardObservable> {
		let publisher = StoryYardPublisher { story: self.clone() };
		Arc::new(publisher)
	}
}

struct StoryYardPublisher<T: Plot> {
	story: Story<T>
}

impl<T: Plot> YardObservable for StoryYardPublisher<T> {
	fn subscribe(&self) -> Result<Receiver<ArcYard>, Box<dyn Error>> {
		let visions = self.story.visions(rand::random())?;
		let (tx_yard, rx_yard) = sync_channel::<ArcYard>(64);
		let link = self.story.link();
		thread::spawn(move || {
			let mut done = false;
			while !done {
				let vision = visions.recv().unwrap();
				if let Some(yard) = T::yard(&vision, &link) {
					if let Err(_) = tx_yard.send(yard) {
						done = true;
					}
				};
			}
		});
		Ok(rx_yard)
	}
}
