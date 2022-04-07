use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::thread;

use crate::{ArcYard, Spark, story_verse};
use crate::story_verse::story_box::StoryBoxAction;

#[derive(Clone)]
pub struct StoryVerse {
	link: Sender<StoryVerseAction>,
	main_story_id: usize,
}

impl StoryVerse {
	pub fn build(spark: impl Spark + Send + 'static) -> Self {
		let (link, main_story_id) = story_verse::connect(spark);
		StoryVerse { link, main_story_id }
	}
	pub fn main_story_id(&self) -> usize { self.main_story_id }
	pub fn link(&self) -> &Sender<StoryVerseAction> { &self.link }
	pub fn add_watcher(&self, watcher_id: usize, yards_link: Sender<Option<ArcYard>>) {
		let action = StoryVerseAction::WatchMain { watcher_id, yards_link };
		self.link.send(action).expect("add watcher to story-verse");
	}
	pub fn end_watcher(&self, watcher_id: usize) {
		let action = StoryVerseAction::EndWatchMain { watcher_id };
		self.link.send(action).expect("end watcher in story-verse");
	}
}

pub enum StoryVerseAction {
	WatchMain { watcher_id: usize, yards_link: Sender<Option<ArcYard>> },
	EndWatchMain { watcher_id: usize },
}

fn connect(spark: impl Spark + Send + 'static) -> (Sender<StoryVerseAction>, usize) {
	const MAIN_BOX_ID: usize = 0;
	let (verse_link, action_source) = channel::<StoryVerseAction>();
	thread::spawn(move || {
		struct State {
			box_links: HashMap<usize, Sender<story_box::StoryBoxAction>>,
			main_watchers: HashMap<usize, Sender<Option<ArcYard>>>,
		}
		impl State {
			pub fn main_box_link(&self) -> &Sender<story_box::StoryBoxAction> { self.box_links.get(&MAIN_BOX_ID).expect("main box") }
		}

		let mut state = State { box_links: HashMap::new(), main_watchers: HashMap::new() };
		state.box_links.insert(MAIN_BOX_ID, story_box::connect(spark));
		for action in action_source {
			match action {
				StoryVerseAction::WatchMain { watcher_id, yards_link } => {
					state.main_watchers.insert(watcher_id, yards_link.clone());
					state.main_box_link().send(StoryBoxAction::AddWatcher { watcher_id, yard_link: yards_link.clone() }).ok();
				}
				StoryVerseAction::EndWatchMain { watcher_id } => {
					state.main_watchers.remove(&watcher_id);
					state.main_box_link().send(StoryBoxAction::EndWatcher { watcher_id }).ok();
				}
			}
		}
	});
	(verse_link, MAIN_BOX_ID)
}

mod story_box;