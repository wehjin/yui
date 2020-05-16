use std::error::Error;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, sync_channel};
use std::thread;

use crate::{ArcYard, Story, Wheel};
use crate::yard::{YardObservable, YardObservableSource};

impl<T: Wheel> YardObservableSource for Story<T> {
	fn yards(&self) -> Arc<dyn YardObservable> {
		let publisher = StoryYardPublisher { story: self.clone() };
		Arc::new(publisher)
	}
}

struct StoryYardPublisher<T: Wheel> {
	story: Story<T>
}

impl<T: Wheel> YardObservable for StoryYardPublisher<T> {
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
