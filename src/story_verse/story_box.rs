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
}

impl Sendable for StoryBoxAction {}

pub fn connect(spark: impl Spark + Send + 'static) -> Sender<StoryBoxAction> {
	let (link, actions) = channel::<StoryBoxAction>();
	thread::spawn(move || {
		let mut state = State { yard: yard::empty(), watchers: HashMap::new(), stopped: false };
		for action in actions {
			match action {
				StoryBoxAction::SetYard(yard) => state.set_yard(yard),
				StoryBoxAction::SetStopped => state.set_stopped(),
				StoryBoxAction::AddWatcher { watcher_id, yard_link } => state.add_watcher(watcher_id, yard_link),
				StoryBoxAction::EndWatcher { watcher_id } => state.end_watcher(watcher_id),
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
	pub fn set_yard(&mut self, yard: ArcYard) {
		if !self.stopped {
			self.yard = yard.clone();
			self.inform_watchers(Some(yard))
		}
	}
	pub fn set_stopped(&mut self) {
		if !self.stopped {
			self.stopped = true;
			self.inform_watchers(None);
		}
	}
	pub fn add_watcher(&mut self, watcher_id: usize, yard_link: Sender<Option<ArcYard>>) {
		let option = if self.stopped { None } else { Some(self.yard.clone()) };
		if yard_link.send(option).is_ok() {
			self.watchers.insert(watcher_id, yard_link);
		}
	}
	pub fn end_watcher(&mut self, watcher_id: usize) {
		self.watchers.remove(&watcher_id);
	}
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
	let story = story::spark(spark, Some(MinEdge::new()), None);
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
