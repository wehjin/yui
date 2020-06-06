use std::error::Error;
use std::sync::mpsc::Receiver;

use crate::{ArcYard, Spark, Story};
use crate::yard::YardPublisher;

pub(crate) struct StoryYardPublisher<S: Spark> {
	pub story: Story<S>
}

impl<Sprk> YardPublisher for StoryYardPublisher<Sprk> where Sprk: Spark + Sync + Send + 'static {
	fn subscribe(&self) -> Result<Receiver<ArcYard>, Box<dyn Error>> { self.story.subscribe() }
}
