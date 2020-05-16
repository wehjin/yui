use std::error::Error;

use crate::{Projector, Wheel};
use crate::app::yard_stack::YardStack;
use crate::yard::YardObservableSource;

pub use self::edge::*;

pub(crate) mod yard_stack;

mod edge;

pub fn run<W: Wheel>() -> Result<(), Box<dyn Error>> {
	let front_story = {
		let story = YardStack::launch(None, None);
		let first_action = {
			let edge = Edge::new(story.link());
			let app_story = W::launch(Some(edge), None);
			yard_stack::Action::PushFront(app_story.yards())
		};
		story.link().send(first_action);
		story
	};
	Projector::project_yards(front_story.yards().subscribe()?)
}
