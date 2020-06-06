use std::sync::Arc;

use crate::{Link, Spark, Story};
use crate::app::pub_stack;

pub struct Edge {
	link: Link<pub_stack::Action>
}

impl Clone for Edge {
	fn clone(&self) -> Self { Edge { link: self.link.clone() } }
}

impl Edge {
	pub fn start_dialog<S>(&self, spark: S, report_link: Link<S::Report>) -> Story<S>
		where S: Spark + Sync + Send + 'static
	{
		let story = spark.spark(Some(self.clone()), Some(report_link));
		self.link.send(pub_stack::Action::Push(Arc::new(story.clone())));
		story
	}

	pub fn end_dialog(&self) {
		self.link.send(pub_stack::Action::Pop);
	}

	pub(crate) fn new(link: Link<pub_stack::Action>) -> Self { Edge { link } }
}
