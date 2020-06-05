use crate::{Link, Spark, Story};
use crate::app::yard_stack;
use crate::yard::YardPublisherSource;

pub struct Edge {
	link: Link<yard_stack::Action>
}

impl Clone for Edge {
	fn clone(&self) -> Self { Edge { link: self.link.clone() } }
}

impl Edge {
	pub fn start_dialog<S>(&self, spark: S, report_link: Link<S::Report>) -> Story<S>
		where S: Spark + Sync + Send + 'static
	{
		let story = spark.spark(Some(self.clone()), Some(report_link));
		self.link.send(yard_stack::Action::PushFront(story.yard_publisher()));
		story
	}

	pub fn end_dialog(&self) {
		self.link.send(yard_stack::Action::PopFront);
	}

	pub(crate) fn new(link: Link<yard_stack::Action>) -> Self { Edge { link } }
}
