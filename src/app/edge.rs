use crate::{Link, Story, Wheel};
use crate::app::yard_stack;
use crate::yard::YardObservableSource;

pub struct Edge {
	link: Link<yard_stack::Action>
}

impl Clone for Edge {
	fn clone(&self) -> Self { Edge { link: self.link.clone() } }
}

impl Edge {
	pub fn start_dialog<W: Wheel>(&self) -> Story<W> {
		let story = W::launch(Some(self.clone()), None);
		self.link.send(yard_stack::Action::PushFront(story.yards()));
		story
	}

	pub fn end_dialog(&self) {
		self.link.send(yard_stack::Action::PopFront);
	}

	pub(crate) fn new(link: Link<yard_stack::Action>) -> Self { Edge { link } }
}
