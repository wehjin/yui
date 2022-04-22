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
use crate::StoryVerseAction::{GetStats, StartYardsFeed};

pub mod story_id;

pub mod sub_story;
pub mod super_story;
pub mod dialog_story;
pub(crate) mod story_box;

#[derive(Debug, Clone)]
pub struct StoryVerseStats {
	pub story_count: usize,
}

#[derive(Clone)]
pub struct StoryVerse {
	story_verse_link: Sender<StoryVerseAction>,
	root_story_id: StoryId,
}

impl StoryVerse {
	pub fn build<S: Spark>(spark: S, story_id: StoryId) -> (StoryVerse, SenderLink<S::Action>) where S: Send + 'static {
		let (story_verse_link, root_story_id) = story_verse::connect();
		let main_story_id = story_id;
		let (main_story_box, main_sender) = story_box::connect(spark, None, main_story_id, story_verse_link.clone());
		story_verse_link.send(StoryVerseAction::AddStackStoryBox(main_story_box, main_story_id)).expect("Add main story box to stack");
		(StoryVerse { story_verse_link, root_story_id }, main_sender)
	}

	pub fn read_stats(&self) -> StoryVerseStats {
		let (stats_link, stats_read) = channel::<StoryVerseStats>();
		self.story_verse_link.send(GetStats(stats_link)).expect("send stats request");
		stats_read.recv().expect("receive stats")
	}
	pub fn start_yards(&self) -> Receiver<(StoryId, Option<ArcYard>)> {
		let (feed_link, feed_source) = channel();
		self.story_verse_link.send(StartYardsFeed(feed_link)).expect("send feed request");
		feed_source
	}
	pub fn root_story_id(&self) -> StoryId { self.root_story_id }
}

fn notify_stack_when_story_stops(story_verse_link: &Sender<StoryVerseAction>, story_stack_link: &SenderLink<StoryStackAction>) {
	let (send_story_stopped, receive_story_stopped) = channel();
	story_verse_link.send(StoryVerseAction::StartStoryStopFeed(send_story_stopped)).expect("Send StartStoryStopFeed");
	let root_sender = story_stack_link.clone();
	thread::spawn(move || {
		for stopped_story_id in receive_story_stopped {
			root_sender.send(StoryStackAction::PopStory(stopped_story_id));
		}
	});
}

pub enum StoryVerseAction {
	GetStats(Sender<StoryVerseStats>),
	StartYardsFeed(Sender<(StoryId, Option<ArcYard>)>),
	AddStoryBox(Sender<StoryBoxAction>, StoryId),
	AddStackStoryBox(Sender<StoryBoxAction>, StoryId),
	StoryBoxStopped(StoryId),
	StartStoryStopFeed(Sender<StoryId>),
	StoryBoxUpdate(StoryId, Option<ArcYard>),
}

fn connect() -> (Sender<StoryVerseAction>, StoryId) {
	let (story_verse_link, action_source) = channel();

	let stack_story_id = StoryId::random();
	let (stack_story_box, stack_link) = story_box::connect(StoryStack {}, None, stack_story_id, story_verse_link.clone());
	story_verse_link.send(StoryVerseAction::AddStoryBox(stack_story_box, stack_story_id)).expect("Add root story box");
	notify_stack_when_story_stops(&story_verse_link, &stack_link);

	let own_link = story_verse_link.clone();
	thread::spawn(move || {
		let mut latest_yards: HashMap<StoryId, Option<ArcYard>> = HashMap::new();
		let mut story_box_links: HashMap<StoryId, Sender<StoryBoxAction>> = HashMap::new();
		let mut yard_feed_links: HashMap<u64, Sender<(StoryId, Option<ArcYard>)>> = HashMap::new();
		let mut story_stop_feed_links: HashMap<u64, Sender<StoryId>> = HashMap::new();
		for action in action_source {
			match action {
				StoryVerseAction::GetStats(stats_link) => {
					let count_excluding_root = story_box_links.len() - 1;
					let stats = StoryVerseStats { story_count: count_excluding_root };
					stats_link.send(stats).expect("send stats response");
				}
				StoryVerseAction::StartYardsFeed(yards_link) => {
					if push_yards_to_feed(&latest_yards, &yards_link).is_ok() {
						yard_feed_links.insert(random(), yards_link);
					}
				}
				StoryVerseAction::AddStoryBox(story_box, story_id) => {
					story_box_links.insert(story_id, story_box.clone());
					start_story_box_feed(&story_box, own_link.clone());
				}
				StoryVerseAction::AddStackStoryBox(story_box, story_id) => {
					own_link.send(StoryVerseAction::AddStoryBox(story_box, story_id)).expect("Add story box");
					stack_link.send(StoryStackAction::PushStory(story_id));
				}
				StoryVerseAction::StoryBoxStopped(story_id) => {
					info!("STORY VERSE STORY BOX STOPPED: {:?}", story_id);
					story_box_links.remove(&story_id);
					latest_yards.remove(&story_id);
					push_yard_to_feeds(story_id, None, &mut yard_feed_links);
					push_stop_to_feeds(story_id, &mut story_stop_feed_links);
				}
				StoryVerseAction::StartStoryStopFeed(story_stop_link) => {
					story_stop_feed_links.insert(random(), story_stop_link);
				}
				StoryVerseAction::StoryBoxUpdate(story_id, story_yard) => {
					latest_yards.insert(story_id, story_yard.clone());
					push_yard_to_feeds(story_id, story_yard, &mut yard_feed_links)
				}
			}
		}
	});
	(story_verse_link, stack_story_id)
}

fn push_yards_to_feed(story_yards: &HashMap<StoryId, Option<ArcYard>>, yards_link: &Sender<(StoryId, Option<ArcYard>)>) -> Result<(), Box<dyn Error>> {
	for (story_id, story_yard) in story_yards {
		yards_link.send((story_id.clone(), story_yard.clone()))?;
	}
	Ok(())
}

fn push_yard_to_feeds(story_id: StoryId, story_yard: Option<ArcYard>, active_feed_links: &mut HashMap<u64, Sender<(StoryId, Option<ArcYard>)>>) {
	let value = (story_id, story_yard);
	push_value_to_feeds(value, active_feed_links);
}

fn push_stop_to_feeds(story_id: StoryId, feed_links: &mut HashMap<u64, Sender<StoryId>>) {
	push_value_to_feeds(story_id, feed_links)
}

fn push_value_to_feeds<T: Clone>(value: T, feed_links: &mut HashMap<u64, Sender<T>>) {
	let mut dead_ids = HashSet::new();
	let feed_ids = feed_links.keys().into_iter().cloned().collect::<Vec<_>>();
	for feed_id in feed_ids {
		let feed_link = feed_links.index(&feed_id);
		if feed_link.send(value.clone()).is_err() {
			dead_ids.insert(feed_id);
		}
	}
	for feed_id in dead_ids {
		feed_links.remove(&feed_id);
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
