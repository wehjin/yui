use std::collections::HashMap;
use std::ops::Index;
use std::sync::mpsc::{channel, Sender};
use std::thread;

use crate::{ArcYard, Link, pod_verse, Sendable, StoryVerse, Trigger};
use crate::pod::link_pod::LinkPod;
use crate::pod::Pod;
use crate::pod::yard::YardPod;
use crate::spot::spot_table::SpotTable;

#[derive(Clone)]
pub struct PodVerse {
	link: Sender<PodVerseAction>,
}

impl PodVerse {
	pub fn build(story_verse: &StoryVerse) -> Self {
		let link = pod_verse::connect(story_verse);
		PodVerse { link }
	}
	pub fn link(&self) -> &Sender<PodVerseAction> { &self.link }
	pub fn to_link_pod(&self, screen_refresh_trigger: Trigger) -> LinkPod {
		LinkPod::new(self.link.clone(), screen_refresh_trigger)
	}
	pub fn set_done_trigger(&self, trigger: Sender<()>) {
		self.link.send(PodVerseAction::SetDoneTrigger(trigger)).expect("set-done-trigger");
	}
}

#[derive(Clone)]
pub enum PodVerseAction {
	UpdatePod { story_id: usize, yard: Option<ArcYard> },
	Refresh,
	SetWidthHeight { width: i32, height: i32 },
	Edit(EditAction),
	LayoutAndRender(Sender<SpotTable>),
	SetDoneTrigger(Sender<()>),
	SetScreenRefreshTrigger(Trigger),
}

#[derive(Debug, Clone)]
pub enum EditAction {
	InsertSpace,
	InsertChar(char),
	MoveFocus(MoveDirection),
}

#[derive(Debug, Clone)]
pub enum MoveDirection {
	Up,
	Down,
	Left,
	Right,
}

impl Sendable for PodVerseAction {}

fn connect(story_verse: &StoryVerse) -> Sender<PodVerseAction> {
	let (pod_verse_link, action_source) = channel::<PodVerseAction>();
	let pod_verse_id = rand::random::<usize>();
	let own_actions = pod_verse_link.clone();
	let main_story_id = story_verse.main_story_id();
	thread::spawn(move || {
		let mut state = State {
			main_story_id,
			pods: HashMap::new(),
			refresh_trigger: PodVerseAction::Refresh.into_trigger(&own_actions),
			done_trigger: None,
			screen_refresh_trigger: None,
		};
		for action in action_source {
			match action {
				PodVerseAction::UpdatePod { story_id, yard } => {
					if let Some(yard) = yard {
						let mut pod = state.pods.remove(&story_id).unwrap_or_else(|| YardPod::new(state.refresh_trigger.clone()));
						pod.set_yard(yard);
						state.pods.insert(story_id, pod);
						state.refresh_pod_verse();
					} else {
						if let Some(done_trigger) = &state.done_trigger {
							done_trigger.send(()).expect("send done signal");
						}
					}
				}
				PodVerseAction::Refresh => {
					if let Some(trigger) = &state.screen_refresh_trigger {
						trigger.send(());
					}
				}
				PodVerseAction::SetWidthHeight { width, height } => {
					state.main_pod_mut().set_width_height((width, height))
				}
				PodVerseAction::Edit(edit) => {
					match edit {
						EditAction::InsertSpace => state.main_pod().insert_space(),
						EditAction::InsertChar(c) => state.main_pod().insert_char(c),
						EditAction::MoveFocus(direction) => match direction {
							MoveDirection::Up => state.main_pod_mut().focus_up(),
							MoveDirection::Down => state.main_pod_mut().focus_down(),
							MoveDirection::Left => state.main_pod_mut().focus_left(),
							MoveDirection::Right => state.main_pod_mut().focus_right(),
						}
					}
					state.refresh_pod_verse();
				}
				PodVerseAction::LayoutAndRender(result) => {
					let spot_table = state.main_pod_mut().layout_and_render().clone();
					result.send(spot_table).expect("send spot-table");
				}
				PodVerseAction::SetDoneTrigger(trigger) => {
					state.done_trigger = Some(trigger)
				}
				PodVerseAction::SetScreenRefreshTrigger(trigger) => {
					state.screen_refresh_trigger = Some(trigger);
				}
			}
		}
	});
	connect_story_verse(story_verse, pod_verse_id, pod_verse_link.clone());
	pod_verse_link
}

struct State {
	main_story_id: usize,
	pods: HashMap<usize, YardPod>,
	refresh_trigger: Trigger,
	done_trigger: Option<Sender<()>>,
	screen_refresh_trigger: Option<Trigger>,
}

impl State {
	pub fn main_pod(&self) -> &impl Pod { self.pods.index(&self.main_story_id) }
	pub fn main_pod_mut(&mut self) -> &mut impl Pod {
		self.pods.get_mut(&self.main_story_id).expect("pod for main story")
	}
	pub fn refresh_pod_verse(&self) {
		self.refresh_trigger.send(());
	}
}

fn connect_story_verse(story_verse: &StoryVerse, pod_verse_id: usize, pod_verse_link: Sender<PodVerseAction>) {
	let (yards_link, yards_source) = channel::<Option<ArcYard>>();
	story_verse.add_watcher(pod_verse_id, yards_link);
	let story_verse = story_verse.clone();
	thread::spawn(move || {
		for yard in yards_source {
			let action = PodVerseAction::UpdatePod { story_id: story_verse.main_story_id(), yard };
			pod_verse_link.send(action).expect("Send UpdatePod to pod-verse");
		}
		story_verse.end_watcher(pod_verse_id);
	});
}
