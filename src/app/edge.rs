use std::sync::mpsc::Sender;

use crate::{Link, SenderLink, Spark, story_box, StoryVerseAction, Trigger};
use crate::dialog_story::DialogStory;
use crate::story_id::StoryId;
use crate::sub_story::SubStory;
use crate::super_story::SuperStory;

#[derive(Debug, Clone)]
pub struct Edge {
	story_id: StoryId,
	end_dialog_trigger: Trigger,
	story_verse_link: Sender<StoryVerseAction>,
}

impl Edge {
	pub fn new(story_id: StoryId, end_dialog_trigger: Trigger, story_verse_link: Sender<StoryVerseAction>) -> Self {
		Edge { story_id, end_dialog_trigger, story_verse_link }
	}
}

impl SuperStory for Edge {
	fn sub_story<S: Spark + Send + 'static>(&self, spark: S, reports_link: Option<SenderLink<S::Report>>) -> SubStory {
		let story_id = self.story_id.sub_id();
		let (story_box_link, _sub_story_link) = story_box::connect(spark, reports_link, story_id, self.story_verse_link.clone());
		self.story_verse_link.send(StoryVerseAction::AddStoryBox(story_box_link, story_id)).expect("Add sub-story box");
		SubStory { story_id }
	}
}

impl Edge {
	pub fn story_id(&self) -> &StoryId { &self.story_id }

	pub fn start_dialog<S: Spark + Send + 'static>(&self, spark: S, report_link: SenderLink<S::Report>) -> DialogStory {
		let story_id = self.story_id.dialog_id();
		let (story_box_link, _dialog_story_link) = story_box::connect(spark, Some(report_link), story_id, self.story_verse_link.clone());
		self.story_verse_link.send(StoryVerseAction::AddStackStoryBox(story_box_link, story_id)).expect("Send AddStackStoryBox");
		DialogStory { story_id }
	}

	pub fn end_dialog(&self) {
		info!("MIN EDGE END DIALOG: {:?}", &self.story_id);
		self.end_dialog_trigger.send(());
	}

	pub fn redraw(&self) {}
}
