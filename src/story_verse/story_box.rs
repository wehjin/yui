use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Sender};
use std::thread;

use crate::{ArcYard, Sendable, SenderLink, Spark, story, StoryVerseAction, yard};
use crate::app::MinEdge;
use crate::story_id::StoryId;
use crate::yard::YardPublisher;

#[derive(Clone)]
pub enum StoryBoxAction {
	SetYard(ArcYard),
	SetStopped,
	AddWatcher { watcher_id: StoryId, yard_link: Sender<Option<ArcYard>> },
	EndWatcher { watcher_id: StoryId },
	EndDialog,
}

impl Sendable for StoryBoxAction {}

pub fn connect<S>(
	spark: S,
	reports_link: Option<SenderLink<S::Report>>,
	story_id: StoryId,
	story_verse_link: Sender<StoryVerseAction>,
) -> Sender<StoryBoxAction> where S: Spark + Send + 'static {
	let (story_box_link, actions) = channel::<StoryBoxAction>();
	let own_actions = story_box_link.clone();
	thread::spawn(move || {
		let mut state = State { yard: yard::empty(), watchers: HashMap::new(), stopped: false };
		for action in actions {
			let option = if state.stopped { None } else { Some(state.yard.clone()) };
			match action {
				StoryBoxAction::SetYard(yard) => if !state.stopped {
					state.yard = yard.clone();
					state.inform_watchers(Some(yard))
				},
				StoryBoxAction::SetStopped => if !state.stopped {
					state.stopped = true;
					state.inform_watchers(None);
				},
				StoryBoxAction::AddWatcher { watcher_id, yard_link } => if yard_link.send(option).is_ok() {
					state.watchers.insert(watcher_id, yard_link);
				},
				StoryBoxAction::EndWatcher { watcher_id } => {
					state.watchers.remove(&watcher_id);
				}
				StoryBoxAction::EndDialog => {
					own_actions.send(StoryBoxAction::SetStopped).expect("set stopped");
				}
			}
		}
	});
	connect_story(spark, reports_link, story_id, story_box_link.clone(), story_verse_link);
	story_box_link
}

struct State {
	yard: ArcYard,
	watchers: HashMap<StoryId, Sender<Option<ArcYard>>>,
	stopped: bool,
}

impl State {
	fn inform_watchers(&mut self, option: Option<ArcYard>) {
		let mut dead_ids: HashSet<StoryId> = HashSet::new();
		for (id, yard_link) in &self.watchers {
			if yard_link.send(option.clone()).is_err() {
				dead_ids.insert(*id);
			}
		}
		for id in dead_ids {
			self.watchers.remove(&id);
		}
	}
}

pub fn connect_story<S>(
	spark: S,
	reports_link: Option<SenderLink<S::Report>>,
	story_id: StoryId,
	story_box_link: Sender<StoryBoxAction>,
	story_verse_link: Sender<StoryVerseAction>,
) where S: Spark + Send + 'static {
	let edge = MinEdge::new(
		story_id,
		StoryBoxAction::EndDialog.into_trigger(&story_box_link),
		story_verse_link,
	);
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
}
