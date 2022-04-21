use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::ops::Index;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use rand::random;

use crate::{ArcYard, Link, SenderLink, Spark, story_verse};
use crate::app::pub_stack::story_stack::{StoryStack, StoryStackAction};
use crate::story_id::StoryId;
use crate::story_verse::story_box::StoryBoxAction;
use crate::StoryVerseAction::{GetStats, StartFeed};

pub mod story_id;

pub mod sub_story;
pub mod super_story;
pub(crate) mod story_box;

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
		let own_link = story_verse::connect();

		let root_story_id = StoryId::random();
		let (root_story_box, root_sender) = story_box::connect(StoryStack {}, None, root_story_id, own_link.clone());
		own_link.send(StoryVerseAction::AddStoryBox(root_story_box, root_story_id)).expect("Add root story box");

		let main_story_id = story_id;
		let (main_story_box, main_sender) = story_box::connect(spark, None, main_story_id, own_link.clone());
		own_link.send(StoryVerseAction::AddStoryBox(main_story_box, main_story_id)).expect("Add main story box");

		root_sender.send(StoryStackAction::PushStory(main_story_id));
		(StoryVerse { story_verse_link: own_link }, main_sender)
	}
	pub fn read_stats(&self) -> StoryVerseStats {
		let (stats_link, stats_read) = channel::<StoryVerseStats>();
		self.story_verse_link.send(GetStats(stats_link)).expect("send stats request");
		stats_read.recv().expect("receive stats")
	}
	pub fn start_yards(&self) -> Receiver<(StoryId, Option<ArcYard>)> {
		let (feed_link, feed_source) = channel();
		self.story_verse_link.send(StartFeed(feed_link)).expect("send feed request");
		feed_source
	}
}

pub enum StoryVerseAction {
	GetStats(Sender<StoryVerseStats>),
	StartFeed(Sender<(StoryId, Option<ArcYard>)>),
	AddStoryBox(Sender<StoryBoxAction>, StoryId),
	StoryBoxUpdate(StoryId, Option<ArcYard>),
}

fn connect() -> Sender<StoryVerseAction> {
	let (story_verse_link, action_source) = channel();
	let own_link = story_verse_link.clone();
	thread::spawn(move || {
		let mut latest_yards: HashMap<StoryId, Option<ArcYard>> = HashMap::new();
		let mut story_box_links: HashMap<StoryId, Sender<StoryBoxAction>> = HashMap::new();
		let mut active_feed_links: HashMap<usize, Sender<(StoryId, Option<ArcYard>)>> = HashMap::new();
		for action in action_source {
			match action {
				StoryVerseAction::GetStats(stats_link) => {
					let count_excluding_root = story_box_links.len() - 1;
					let stats = StoryVerseStats { story_count: count_excluding_root };
					stats_link.send(stats).expect("send stats response");
				}
				StoryVerseAction::StartFeed(yards_link) => {
					if push_yards_to_feed(&latest_yards, &yards_link).is_ok() {
						active_feed_links.insert(random(), yards_link);
					}
				}
				StoryVerseAction::AddStoryBox(story_box, story_id) => {
					story_box_links.insert(story_id, story_box.clone());
					start_story_box_feed(&story_box, own_link.clone());
				}
				StoryVerseAction::StoryBoxUpdate(story_id, story_yard) => {
					latest_yards.insert(story_id, story_yard.clone());
					push_yard_to_feeds(story_id, story_yard, &mut active_feed_links)
				}
			}
		}
	});
	story_verse_link
}

fn push_yards_to_feed(story_yards: &HashMap<StoryId, Option<ArcYard>>, yards_link: &Sender<(StoryId, Option<ArcYard>)>) -> Result<(), Box<dyn Error>> {
	for (story_id, story_yard) in story_yards {
		yards_link.send((story_id.clone(), story_yard.clone()))?;
	}
	Ok(())
}

fn push_yard_to_feeds(story_id: StoryId, story_yard: Option<ArcYard>, active_feed_links: &mut HashMap<usize, Sender<(StoryId, Option<ArcYard>)>>) {
	let mut dead_ids = HashSet::new();
	let feed_ids = active_feed_links.keys().into_iter().cloned().collect::<Vec<_>>();
	for feed_id in feed_ids {
		let feed_link = active_feed_links.index(&feed_id);
		if feed_link.send((story_id, story_yard.clone())).is_err() {
			dead_ids.insert(feed_id);
		}
	}
	for feed_id in dead_ids {
		active_feed_links.remove(&feed_id);
	}
}

fn start_story_box_feed(story_box_link: &Sender<StoryBoxAction>, story_verse_link: Sender<StoryVerseAction>) {
	let (feed_link, feed_source) = channel();
	story_box_link.send(StoryBoxAction::StartFeed(feed_link)).expect("send start-feed to story-box");
	thread::spawn(move || {
		for (story_id, story_yard) in feed_source {
			story_verse_link.send(StoryVerseAction::StoryBoxUpdate(story_id, story_yard)).expect("send update-story-yard to story-verse");
		}
	});
}
