use crate::{Link, SenderLink, Spark, Story, Trigger};
use crate::app::pub_stack;
use crate::prelude::story;

pub trait Edge {
	fn start_dialog<S: Spark + Send + 'static>(&self, spark: S, report_link: SenderLink<S::Report>) -> Story<S>;
	fn end_dialog(&self);
	fn redraw(&self);
}

#[derive(Debug, Clone)]
pub struct MinEdge {
	end_dialog_trigger: Trigger,
}

impl MinEdge {
	pub fn new(end_dialog_trigger: Trigger) -> Self {
		MinEdge { end_dialog_trigger }
	}
}

impl Edge for MinEdge {
	fn start_dialog<S: Spark + Send + 'static>(&self, _spark: S, _report_link: SenderLink<S::Report>) -> Story<S> {
		unimplemented!()
	}
	fn end_dialog(&self) {
		self.end_dialog_trigger.send(());
	}
	fn redraw(&self) {}
}

pub struct SimpleEdge {
	redraw_trigger: Trigger,
}

impl SimpleEdge {
	pub fn new(redraw_trigger: Trigger) -> Self { SimpleEdge { redraw_trigger } }
}

impl Clone for SimpleEdge {
	fn clone(&self) -> Self { SimpleEdge { redraw_trigger: self.redraw_trigger.clone() } }
}

impl Edge for SimpleEdge {
	fn start_dialog<S>(&self, _spark: S, _report_link: SenderLink<S::Report>) -> Story<S> where S: Spark + Send + 'static {
		todo!()
	}

	fn end_dialog(&self) {
		todo!()
	}

	fn redraw(&self) {
		self.redraw_trigger.send(());
	}
}

pub struct AppEdge {
	link: SenderLink<pub_stack::Action>,
	redraw: SenderLink<()>,
}

impl Clone for AppEdge {
	fn clone(&self) -> Self {
		AppEdge { link: self.link.clone(), redraw: self.redraw.clone() }
	}
}

impl AppEdge {
	pub(crate) fn new(link: SenderLink<pub_stack::Action>, redraw: SenderLink<()>) -> Self {
		AppEdge { link, redraw }
	}
}

impl Edge for AppEdge {
	fn start_dialog<S>(&self, spark: S, report_link: SenderLink<S::Report>) -> Story<S> where S: Spark + Send + 'static {
		let story = story::spark(spark, Some(self.clone()), Some(report_link));
		self.link.send(pub_stack::Action::Push(story.connect()));
		story
	}

	fn end_dialog(&self) {
		self.link.send(pub_stack::Action::Pop);
	}


	fn redraw(&self) {
		self.redraw.send(())
	}
}
