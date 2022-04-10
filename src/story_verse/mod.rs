use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::thread;

use crate::{ArcYard, SenderLink, Spark, story_verse};
use crate::story_id::StoryId;
use crate::story_verse::story_box::StoryBoxAction;
use crate::StoryVerseAction::GetStats;

pub mod story_id;

pub mod sub_story;
pub mod super_story;
pub(crate) mod story_box;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub struct StoryVerseStats {
	pub story_count: usize,
}

#[derive(Clone)]
pub struct StoryVerse {
	story_verse_link: Sender<StoryVerseAction>,
}

impl StoryVerse {
	pub fn build<S: Spark>(spark: S, story_id: StoryId) -> (StoryVerse, SenderLink<S::Action>) where S: Send + 'static {
		let story_verse_link = story_verse::connect();
		let (story_box_link, story_link) = story_box::connect(spark, None, story_id, story_verse_link.clone());
		story_verse_link.send(StoryVerseAction::AddStoryBox(story_box_link, story_id)).expect("Add main story box");
		(StoryVerse { story_verse_link }, story_link)
	}
	pub fn add_watcher(&self, watcher_id: StoryId, yards_link: Sender<Option<ArcYard>>) {
		let action = StoryVerseAction::WatchMain { watcher_id, yards_link };
		self.story_verse_link.send(action).expect("add watcher to story-verse");
	}
	pub fn end_watcher(&self, watcher_id: StoryId) {
		let action = StoryVerseAction::EndWatchMain { watcher_id };
		self.story_verse_link.send(action).expect("end watcher in story-verse");
	}
	pub fn read_stats(&self) -> StoryVerseStats {
		let (stats_link, stats_read) = channel::<StoryVerseStats>();
		self.story_verse_link.send(GetStats(stats_link)).expect("send stats request");
		stats_read.recv().expect("receive stats")
	}
}

pub enum StoryVerseAction {
	WatchMain { watcher_id: StoryId, yards_link: Sender<Option<ArcYard>> },
	EndWatchMain { watcher_id: StoryId },
	AddStoryBox(Sender<StoryBoxAction>, StoryId),
	GetStats(Sender<StoryVerseStats>),
}

fn connect() -> Sender<StoryVerseAction> {
	let (story_verse_link, action_source) = channel();
	let main_story_id = StoryId::new(MAIN_STORY);
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
				StoryVerseAction::GetStats(stats_link) => {
					let stats = StoryVerseStats { story_count: state.story_box_links.len() };
					stats_link.send(stats).expect("send stats response");
				}
			}
		}
	});
	story_verse_link
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

const MAIN_STORY: usize = 0;
