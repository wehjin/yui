use std::sync::Arc;

use crate::{SyncLink, Spark, Story, Link};
use crate::app::pub_stack;
use crate::prelude::story;

pub struct Edge {
	link: SyncLink<pub_stack::Action>
}

impl Clone for Edge {
	fn clone(&self) -> Self { Edge { link: self.link.clone() } }
}

impl Edge {
	pub fn start_dialog<S>(&self, spark: S, report_link: SyncLink<S::Report>) -> Story<S>
		where S: Spark + Sync + Send + 'static
	{
		let story = story::spark(spark, Some(self.clone()), Some(report_link));
		self.link.send(pub_stack::Action::Push(Arc::new(story.clone())));
		story
	}

	pub fn end_dialog(&self) {
		self.link.send(pub_stack::Action::Pop);
	}

	pub(crate) fn new(link: SyncLink<pub_stack::Action>) -> Self { Edge { link } }
}
