use crate::{Link, SenderLink, Spark, Story};
use crate::app::pub_stack;
use crate::prelude::story;

pub struct Edge {
	link: SenderLink<pub_stack::Action>,
	redraw: SenderLink<()>,
}

impl Clone for Edge {
	fn clone(&self) -> Self {
		Edge { link: self.link.clone(), redraw: self.redraw.clone() }
	}
}

impl Edge {
	pub fn start_dialog<S>(&self, spark: S, report_link: SenderLink<S::Report>) -> Story<S>
		where S: Spark + Send + 'static
	{
		let story = story::spark(spark, Some(self.clone()), Some(report_link));
		self.link.send(pub_stack::Action::Push(story.connect()));
		story
	}

	pub fn end_dialog(&self) {
		self.link.send(pub_stack::Action::Pop);
	}


	pub fn redraw(&self) {
		self.redraw.send(())
	}

	pub(crate) fn new(link: SenderLink<pub_stack::Action>, redraw: SenderLink<()>) -> Self {
		Edge { link, redraw }
	}
}
