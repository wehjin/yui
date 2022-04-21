use std::sync::mpsc::{channel, Sender};
use std::thread;

use crate::{ArcYard, Bounds, Link, pod_verse, Sendable, StoryVerse, Trigger};
use crate::pod::link_pod::MainPod;
use crate::pod::Pod;
use crate::pod_verse::tree::PodTree;
use crate::spot::spot_table::SpotTable;
use crate::story_id::StoryId;

pub mod tree;

#[derive(Clone)]
pub struct PodVerse {
	pod_verse_link: Sender<PodVerseAction>,
}

impl PodVerse {
	pub fn build(story_verse: &StoryVerse) -> Self {
		let link = pod_verse::connect(story_verse);
		PodVerse { pod_verse_link: link }
	}
	pub fn to_main_pod(&self, screen_refresh_trigger: Trigger) -> MainPod {
		MainPod::new(self.pod_verse_link.clone(), screen_refresh_trigger)
	}
	pub fn set_done_trigger(&self, trigger: Sender<()>) {
		self.pod_verse_link.send(PodVerseAction::SetDoneTrigger(trigger)).expect("set-done-trigger");
	}
	pub fn read_pod_count(&self) -> usize {
		let (response_link, response_source) = channel();
		self.pod_verse_link.send(PodVerseAction::GetPodCount(response_link)).expect("send pod-count request");
		response_source.recv().expect("receive pod-count response")
	}
}

#[derive(Clone)]
pub enum PodVerseAction {
	SetScreenRefreshTrigger(Trigger),
	Refresh,
	FullRefresh,
	YardUpdate { story_id: StoryId, story_yard: Option<ArcYard> },
	SetWidthHeight { width: i32, height: i32 },
	Edit(EditAction),
	ReadSpotTable(Sender<Option<SpotTable>>),
	SetDoneTrigger(Sender<()>),
	GetPodCount(Sender<usize>),
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
	let own_actions = pod_verse_link.clone();
	let root_story_id = story_verse.root_story_id();
	thread::spawn(move || {
		let refresh_trigger = PodVerseAction::FullRefresh.into_trigger(&own_actions);
		let mut pod_tree = PodTree::new(root_story_id, refresh_trigger.clone());
		let mut screen_refresh_trigger: Option<Trigger> = None;
		let mut done_trigger: Option<Sender<()>> = None;
		for action in action_source {
			match action {
				PodVerseAction::SetScreenRefreshTrigger(trigger) => {
					screen_refresh_trigger = Some(trigger);
				}
				PodVerseAction::Refresh => {
					info!("PodVerseRefresh");
					if let Some(trigger) = &screen_refresh_trigger { trigger.send(()); }
				}
				PodVerseAction::FullRefresh => {
					info!("FullRefresh");
					pod_tree.redraw();
					own_actions.send(PodVerseAction::Refresh).expect("send refresh");
				}
				PodVerseAction::GetPodCount(response_link) => {
					let count = pod_tree.layout_count();
					response_link.send(count).expect("Send pod count");
				}
				PodVerseAction::SetDoneTrigger(trigger) => {
					done_trigger = Some(trigger);
				}
				PodVerseAction::SetWidthHeight { width, height } => {
					pod_tree.set_bounds(Bounds::new(width, height));
					own_actions.send(PodVerseAction::Refresh).expect("send refresh");
				}
				PodVerseAction::YardUpdate { story_id, story_yard: yard } => {
					if yard.is_none() && story_id == root_story_id && done_trigger.is_some() {
						done_trigger.clone().unwrap().send(()).expect("send done");
					} else {
						pod_tree.set_story_yard(story_id, yard);
						own_actions.send(PodVerseAction::Refresh).expect("send refresh");
					}
				}
				PodVerseAction::Edit(edit) => {
					match edit {
						EditAction::InsertSpace => pod_tree.insert_space(),
						EditAction::InsertChar(c) => pod_tree.insert_char(c),
						EditAction::MoveFocus(direction) => match direction {
							MoveDirection::Up => pod_tree.focus_up(),
							MoveDirection::Down => pod_tree.focus_down(),
							MoveDirection::Left => pod_tree.focus_left(),
							MoveDirection::Right => pod_tree.focus_right(),
						}
					}
					own_actions.send(PodVerseAction::FullRefresh).expect("send refresh");
				}
				PodVerseAction::ReadSpotTable(result) => {
					let spot_table = Some(pod_tree.to_spot_table());
					result.send(spot_table).expect("send spot-table");
				}
			}
		}
	});
	connect_story_verse(story_verse, pod_verse_link.clone());
	pod_verse_link
}

fn connect_story_verse(story_verse: &StoryVerse, pod_verse_link: Sender<PodVerseAction>) {
	let yards_source = story_verse.start_yards();
	thread::spawn(move || {
		for (story_id, story_yard) in yards_source {
			let action = PodVerseAction::YardUpdate { story_id, story_yard };
			pod_verse_link.send(action).expect("send update-pod to pod-verse");
		}
	});
}
