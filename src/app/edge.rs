use std::sync::mpsc::Sender;

use crate::{Link, SenderLink, Spark, Story, story_box, StoryVerseAction, Trigger};
use crate::app::pub_stack;
use crate::prelude::story;
use crate::story_id::StoryId;
use crate::sub_story::SubStory;
use crate::super_story::SuperStory;

pub trait Edge: SuperStory {
	fn story_id(&self) -> &StoryId;
	fn start_dialog<S: Spark + Send + 'static>(&self, spark: S, report_link: SenderLink<S::Report>) -> Story<S>;
	fn end_dialog(&self);
	fn redraw(&self);
}

#[derive(Debug, Clone)]
pub struct MinEdge {
	story_id: StoryId,
	end_dialog_trigger: Trigger,
	story_verse_link: Sender<StoryVerseAction>,
}

impl MinEdge {
	pub fn new(story_id: StoryId, end_dialog_trigger: Trigger, story_verse_link: Sender<StoryVerseAction>) -> Self {
		MinEdge { story_id, end_dialog_trigger, story_verse_link }
	}
}

impl SuperStory for MinEdge {
	fn sub_story<S: Spark + Send + 'static>(&self, spark: S, reports_link: Option<SenderLink<S::Report>>) -> SubStory {
		let story_id = self.story_id.sub_id();
		let (story_box_link, _sub_story_link) = story_box::connect(spark, reports_link, story_id, self.story_verse_link.clone());
		self.story_verse_link.send(StoryVerseAction::AddStoryBox(story_box_link, story_id)).expect("Add sub-story box");
		SubStory { story_id }
	}
}

impl Edge for MinEdge {
	fn story_id(&self) -> &StoryId { &self.story_id }

	fn start_dialog<S: Spark + Send + 'static>(&self, _spark: S, _report_link: SenderLink<S::Report>) -> Story<S> {
		unimplemented!()
	}
	fn end_dialog(&self) {
		info!("MIN EDGE END DIALOG: {:?}", &self.story_id);
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

impl SuperStory for SimpleEdge {
	fn sub_story<S: Spark>(&self, _spark: S, _reports_link: Option<SenderLink<S::Report>>) -> SubStory {
		todo!()
	}
}

impl Edge for SimpleEdge {
	fn story_id(&self) -> &StoryId {
		todo!()
	}

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

impl SuperStory for AppEdge {
	fn sub_story<S: Spark>(&self, _spark: S, _reports_link: Option<SenderLink<S::Report>>) -> SubStory {
		todo!()
	}
}

impl Edge for AppEdge {
	fn story_id(&self) -> &StoryId {
		todo!()
	}

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
