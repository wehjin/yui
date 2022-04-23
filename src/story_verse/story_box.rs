use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Sender};
use std::thread;

use rand::random;

use crate::{ArcYard, Sendable, SenderLink, Spark, story, Story, StoryVerseAction, yard};
use crate::app::Edge;
use crate::story_id::StoryId;
use crate::yard::YardPublisher;

#[derive(Clone)]
pub enum StoryBoxAction {
	SetYard(ArcYard),
	SetStopped,
	StartFeed(Sender<(StoryId, Option<ArcYard>)>),
	EndDialog,
}

impl Sendable for StoryBoxAction {}

pub fn connect<S: Spark>(
	spark: S,
	reports_link: Option<SenderLink<S::Report>>,
	story_id: StoryId,
	story_verse_link: Sender<StoryVerseAction>,
) -> (Sender<StoryBoxAction>, SenderLink<S::Action>) where S: Send + 'static {
	let (story_box_link, actions) = channel::<StoryBoxAction>();
	let own_actions = story_box_link.clone();
	let own_verse_actions = story_verse_link.clone();
	thread::spawn(move || {
		let mut latest_yard: Option<ArcYard> = Some(yard::empty());
		let mut active_feed_links: HashMap<u64, Sender<(StoryId, Option<ArcYard>)>> = HashMap::new();
		for action in actions {
			match action {
				StoryBoxAction::SetYard(yard) => {
					if latest_yard.is_some() {
						latest_yard = Some(yard.clone());
						push_yard(story_id, &latest_yard, &mut active_feed_links);
					}
				}
				StoryBoxAction::SetStopped => {
					if latest_yard.is_some() {
						latest_yard = None;
						push_yard(story_id, &latest_yard, &mut active_feed_links);
					}
					own_verse_actions.send(StoryVerseAction::StoryBoxStopped(story_id)).expect("Send StoryBoxStopped");
					break;
				}
				StoryBoxAction::StartFeed(feed_link) => {
					if feed_link.send((story_id, latest_yard.clone())).is_ok() {
						active_feed_links.insert(random(), feed_link);
					}
				}
				StoryBoxAction::EndDialog => {
					info!("STORY BOX END DIALOG: {:?}", story_id);
					own_actions.send(StoryBoxAction::SetStopped).expect("set stopped");
				}
			}
		}
		info!("STORY BOX THREAD ENDED: {:?}", story_id);
	});
	let story = connect_story(spark, reports_link, story_id, story_box_link.clone(), story_verse_link);
	(story_box_link, story.link())
}

fn push_yard(story_id: StoryId, story_yard: &Option<ArcYard>, feeds: &mut HashMap<u64, Sender<(StoryId, Option<ArcYard>)>>) {
	let mut dead_ids: HashSet<u64> = HashSet::new();
	feeds.iter().for_each(|(feed_id, feed_link)| {
		if feed_link.send((story_id, story_yard.clone())).is_err() {
			dead_ids.insert(*feed_id);
		}
	});
	for id in dead_ids {
		feeds.remove(&id);
	}
}


fn connect_story<S: Spark>(
	spark: S,
	reports_link: Option<SenderLink<S::Report>>,
	story_id: StoryId,
	story_box_link: Sender<StoryBoxAction>,
	story_verse_link: Sender<StoryVerseAction>,
) -> Story<S> where S: Send + 'static {
	let end_dialog_trigger = StoryBoxAction::EndDialog.into_trigger(&story_box_link);
	let edge = Edge::new(story_id, end_dialog_trigger, story_verse_link);
	let story = story::spark(spark, Some(edge), reports_link);
	match story.subscribe() {
		Ok(yard_source) => {
			thread::spawn(move || {
				let mut link_closed = false;
				for yard in yard_source {
					if story_box_link.send(StoryBoxAction::SetYard(yard)).is_err() {
						link_closed = true;
						break;
					}
				}
				if !link_closed {
					story_box_link.send(StoryBoxAction::SetStopped).ok();
				}
			});
		}
		Err(_) => {
			story_box_link.send(StoryBoxAction::SetStopped).ok();
		}
	}
	story
}
