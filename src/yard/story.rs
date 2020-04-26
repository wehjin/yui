use std::error::Error;
use std::sync::mpsc::{Receiver, sync_channel};
use std::thread;

use crate::{ArcYard, Story, Teller};
use crate::yard::{YardObservable, YardObservableSource};

impl<T: Teller + 'static> YardObservableSource for Story<T> {
	fn yards(&self) -> Box<dyn YardObservable> {
		let publisher = StoryYardPublisher { story: self.dup() };
		Box::new(publisher)
	}
}

struct StoryYardPublisher<T: Teller + 'static> {
	story: Story<T>
}

impl<T: Teller + 'static> YardObservable for StoryYardPublisher<T> {
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
