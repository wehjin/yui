use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Sender};
use std::thread;

use crate::{ArcYard, Sendable, Spark, story, yard};
use crate::app::MinEdge;
use crate::yard::YardPublisher;

#[derive(Clone)]
pub enum StoryBoxAction {
	SetYard(ArcYard),
	SetStopped,
	AddWatcher { watcher_id: usize, yard_link: Sender<Option<ArcYard>> },
	EndWatcher { watcher_id: usize },
	EndDialog,
}

impl Sendable for StoryBoxAction {}

pub fn connect(spark: impl Spark + Send + 'static) -> Sender<StoryBoxAction> {
	let (link, actions) = channel::<StoryBoxAction>();
	let own_actions = link.clone();
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
	connect_story(spark, link.clone());
	link
}

struct State {
	yard: ArcYard,
	watchers: HashMap<usize, Sender<Option<ArcYard>>>,
	stopped: bool,
}

impl State {
	fn inform_watchers(&mut self, option: Option<ArcYard>) {
		let mut dead_ids: HashSet<usize> = HashSet::new();
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

fn connect_story(spark: impl Spark + Send + 'static, link: Sender<StoryBoxAction>) {
	let end_dialog_trigger = StoryBoxAction::EndDialog.into_trigger(&link);
	let edge = MinEdge::new(end_dialog_trigger);
	let story = story::spark(spark, Some(edge), None);
	match story.subscribe() {
		Ok(yard_source) => {
			thread::spawn(move || {
				let mut link_closed = false;
				for yard in yard_source {
					if link.send(StoryBoxAction::SetYard(yard)).is_err() {
						link_closed = true;
						break;
					}
				}
				if !link_closed {
					link.send(StoryBoxAction::SetStopped).ok();
				}
			});
		}
		Err(_) => {
			link.send(StoryBoxAction::SetStopped).ok();
		}
	}
}
