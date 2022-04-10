use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::thread;

use crate::{ArcYard, Spark, story_verse};
use crate::story_id::StoryId;
use crate::story_verse::story_box::StoryBoxAction;

pub mod story_id;

pub mod sub_story;
pub mod super_story;
pub(crate) mod story_box;

#[derive(Clone)]
pub struct StoryVerse {
	link: Sender<StoryVerseAction>,
	main_story_id: StoryId,
}

impl StoryVerse {
	pub fn build(spark: impl Spark + Send + 'static) -> Self {
		let (link, main_story_id) = story_verse::connect(spark);
		StoryVerse { link, main_story_id }
	}
	pub fn main_story_id(&self) -> StoryId { self.main_story_id }
	pub fn link(&self) -> &Sender<StoryVerseAction> { &self.link }
	pub fn add_watcher(&self, watcher_id: StoryId, yards_link: Sender<Option<ArcYard>>) {
		let action = StoryVerseAction::WatchMain { watcher_id, yards_link };
		self.link.send(action).expect("add watcher to story-verse");
	}
	pub fn end_watcher(&self, watcher_id: StoryId) {
		let action = StoryVerseAction::EndWatchMain { watcher_id };
		self.link.send(action).expect("end watcher in story-verse");
	}
}

pub enum StoryVerseAction {
	WatchMain { watcher_id: StoryId, yards_link: Sender<Option<ArcYard>> },
	EndWatchMain { watcher_id: StoryId },
	AddStoryBox(Sender<StoryBoxAction>, StoryId),
}

const MAIN_BOX_ID: usize = 0;

fn connect(spark: impl Spark + Send + 'static) -> (Sender<StoryVerseAction>, StoryId) {
	let (story_verse_link, action_source) = channel::<StoryVerseAction>();
	let main_story_id = StoryId::new(MAIN_BOX_ID);
	thread::spawn(move || {
		let mut state = State { story_box_links: HashMap::new(), main_watchers: HashMap::new() };
		for action in action_source {
			match action {
				StoryVerseAction::WatchMain { watcher_id, yards_link } => {
					state.main_watchers.insert(watcher_id, yards_link.clone());
					state.story_box_link(&main_story_id).send(StoryBoxAction::AddWatcher { watcher_id, yard_link: yards_link.clone() }).ok();
				}
				StoryVerseAction::EndWatchMain { watcher_id } => {
					state.main_watchers.remove(&watcher_id);
					state.story_box_link(&main_story_id).send(StoryBoxAction::EndWatcher { watcher_id }).ok();
				}
				StoryVerseAction::AddStoryBox(story_box, story_id) => {
					state.story_box_links.insert(story_id, story_box);
				}
			}
		}
	});
	let main_story_box = story_box::connect(spark, None, main_story_id, story_verse_link.clone());
	story_verse_link.send(StoryVerseAction::AddStoryBox(main_story_box, main_story_id)).expect("Add main story box");
	(story_verse_link, main_story_id)
}

struct State {
	story_box_links: HashMap<StoryId, Sender<story_box::StoryBoxAction>>,
	main_watchers: HashMap<StoryId, Sender<Option<ArcYard>>>,
}

impl State {
	pub fn story_box_link(&self, story_id: &StoryId) -> &Sender<story_box::StoryBoxAction> {
		self.story_box_links.get(story_id).expect("main box")
	}
}

