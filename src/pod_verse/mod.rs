use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::mpsc::{channel, Sender};
use std::thread;

use crate::{ArcYard, Link, pod_verse, Sendable, StoryVerse, Trigger};
use crate::pod::link_pod::MainPod;
use crate::pod::Pod;
use crate::pod::yard::YardPod;
use crate::spot::spot_table::SpotTable;
use crate::story_id::StoryId;

#[derive(Clone)]
pub struct PodVerse {
	pod_verse_link: Sender<PodVerseAction>,
}

impl PodVerse {
	pub fn build(story_verse: &StoryVerse, main_story_id: StoryId) -> Self {
		let link = pod_verse::connect(story_verse, main_story_id);
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
	YardUpdate { story_id: StoryId, story_yard: Option<ArcYard> },
	Refresh,
	SetWidthHeight { width: i32, height: i32 },
	Edit(EditAction),
	SpotTable(Sender<Option<SpotTable>>),
	SetDoneTrigger(Sender<()>),
	SetScreenRefreshTrigger(Trigger),
	GetPodCount(Sender<usize>),
	SetDependencies(StoryId, HashSet<(StoryId, (i32, i32))>),
	SpotTableChanged(StoryId, (i32, i32)),
	Relink(Option<StoryId>),
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

struct PodBank {
	pod_verse_link: Sender<PodVerseAction>,
	main_story_id: StoryId,
	main_size: (i32, i32),
	main_spot_table: SpotTable,
	pods: HashMap<Option<StoryId>, HashMap<(StoryId, (i32, i32)), YardPod>>,
	refresh_trigger: Trigger,
	yards: HashMap<StoryId, ArcYard>,
	done_trigger: Option<Sender<()>>,
	spot_tables: Rc<RefCell<HashMap<(StoryId, (i32, i32)), SpotTable>>>,
}

impl PodBank {
	pub fn new(pod_verse_link: Sender<PodVerseAction>, refresh_trigger: Trigger, main_story_id: StoryId) -> Self {
		let spot_tables = Rc::new(RefCell::new(Default::default()));
		let main_size = (0, 0);
		let main_spot_table = SpotTable::new(main_size.1, main_size.0);
		let pod = YardPod::new(pod_verse_link.clone(), refresh_trigger.clone(), main_story_id, main_size, spot_tables.clone());
		let subs = vec![((main_story_id, main_size), pod)].into_iter().collect::<HashMap<_, _>>();
		let pods = vec![(None, subs)].into_iter().collect::<HashMap<_, _>>();
		PodBank { pod_verse_link, main_story_id, main_size, main_spot_table, pods, refresh_trigger, yards: HashMap::new(), done_trigger: None, spot_tables }
	}
	pub fn pod_count(&self) -> usize {
		self.pods.iter().fold(0, |count, (_, sub_pods)| count + sub_pods.len())
	}
	pub fn set_pods(&mut self, parent_id: Option<StoryId>, sub_pods: HashSet<(StoryId, (i32, i32))>) {
		trace!("SET_PODS: parent: {:?}, pods: {:?}", parent_id, sub_pods);
		let mut old_pods = self.pods.remove(&parent_id).unwrap_or_else(|| HashMap::new());
		let mut new_pods = HashMap::new();
		let mut added = 0;
		for sub_pod_id in sub_pods {
			let pod = old_pods.remove(&sub_pod_id).unwrap_or_else(|| {
				added += 1;
				let mut pod = YardPod::new(
					self.pod_verse_link.clone(),
					self.refresh_trigger.clone(),
					sub_pod_id.0,
					sub_pod_id.1,
					self.spot_tables.clone(),
				);
				if let Some(yard) = self.yards.get(&sub_pod_id.0) {
					pod.set_yard(yard.clone());
				}
				pod
			});
			new_pods.insert(sub_pod_id, pod);
		}
		self.pods.insert(parent_id, new_pods);
	}
	pub fn relink_parents(&self, story_id: StoryId, width_height: (i32, i32)) {
		trace!("RELINK PARENTS {:?}", story_id);
		let sub_id = (story_id, width_height);
		let needs_link = self.pods.iter().filter_map(|(parent, subs)| {
			if subs.contains_key(&sub_id) { Some(parent.clone()) } else { None }
		}).collect::<Vec<_>>();
		for story_id in needs_link {
			self.pod_verse_link.send(PodVerseAction::Relink(story_id)).expect("send re-link");
		}
	}
	pub fn relink(&mut self, story_id: &Option<StoryId>) {
		trace!("RELINK {:?}", story_id);
		if let Some(story_id) = story_id {
			for (_parent, pods) in &mut self.pods {
				for ((pod_story_id, _pod_size), pod) in pods {
					if pod_story_id == story_id {
						pod.link_tables();
					}
				}
			}
		} else {
			let main_sub = (self.main_story_id, (self.main_size));
			let spot_tables = self.spot_tables.borrow();
			let spot_table = spot_tables.get(&main_sub).expect("main spot table");
			self.main_spot_table = spot_table.clone();
			self.pod_verse_link.send(PodVerseAction::Refresh).expect("update screen");
		}
	}
	pub fn to_spot_table(&self) -> Option<SpotTable> {
		(*self.spot_tables).borrow().get(&(self.main_story_id, self.main_size)).cloned()
	}
	pub fn set_done_trigger(&mut self, trigger: Sender<()>) {
		self.done_trigger = Some(trigger);
	}
	pub fn resize(&mut self, width_height: (i32, i32)) {
		trace!("RESIZE: {:?}", width_height);
		self.main_size = width_height;
		self.set_pods(None, vec![(self.main_story_id, width_height)].into_iter().collect::<HashSet<_>>());
	}
	pub fn refill(&mut self, story_id: StoryId, yard: Option<ArcYard>) {
		trace!("REFILL: {:?}, yard:{:?}", story_id, yard.is_some());
		if let Some(yard) = yard {
			self.yards.insert(story_id, yard.clone());
			for (_parent, pods) in &mut self.pods {
				for ((pod_id, _pod_size), pod) in pods {
					if *pod_id == story_id {
						pod.set_yard(yard.clone());
					}
				}
			}
		} else {
			self.yards.remove(&story_id);
			self.pods.remove(&Some(story_id));
		}
	}
	pub fn main_pod_mut(&mut self) -> &mut YardPod {
		let sub_pod_params = (self.main_story_id, self.main_size);
		self.pods.get_mut(&None).map(|subs| subs.get_mut(&sub_pod_params)).flatten().unwrap()
	}
}


fn connect(story_verse: &StoryVerse, main_story_id: StoryId) -> Sender<PodVerseAction> {
	let (pod_verse_link, action_source) = channel::<PodVerseAction>();
	let own_actions = pod_verse_link.clone();
	thread::spawn(move || {
		let mut pod_bank = PodBank::new(
			own_actions.clone(),
			PodVerseAction::Refresh.into_trigger(&own_actions),
			main_story_id,
		);
		let mut screen_refresh_trigger: Option<Trigger> = None;
		for action in action_source {
			match action {
				PodVerseAction::GetPodCount(response_link) => {
					response_link.send(pod_bank.pod_count()).expect("Send pod count");
				}
				PodVerseAction::SetDoneTrigger(trigger) => {
					pod_bank.set_done_trigger(trigger);
				}
				PodVerseAction::SetWidthHeight { width, height } => {
					pod_bank.resize((width, height));
				}
				PodVerseAction::YardUpdate { story_id, story_yard: yard } => {
					pod_bank.refill(story_id, yard);
				}
				PodVerseAction::Refresh => {
					if let Some(trigger) = &screen_refresh_trigger {
						trigger.send(());
					}
				}
				PodVerseAction::SetScreenRefreshTrigger(trigger) => {
					screen_refresh_trigger = Some(trigger);
				}
				PodVerseAction::Edit(edit) => {
					match edit {
						EditAction::InsertSpace => pod_bank.main_pod_mut().insert_space(),
						EditAction::InsertChar(c) => pod_bank.main_pod_mut().insert_char(c),
						EditAction::MoveFocus(direction) => match direction {
							MoveDirection::Up => pod_bank.main_pod_mut().focus_up(),
							MoveDirection::Down => pod_bank.main_pod_mut().focus_down(),
							MoveDirection::Left => pod_bank.main_pod_mut().focus_left(),
							MoveDirection::Right => pod_bank.main_pod_mut().focus_right(),
						}
					}
					own_actions.send(PodVerseAction::Refresh).expect("send refresh");
				}
				PodVerseAction::SpotTable(result) => {
					let spot_table = pod_bank.to_spot_table();
					result.send(spot_table).expect("send spot-table");
				}
				PodVerseAction::SetDependencies(parent_id, sub_pods) => {
					pod_bank.set_pods(Some(parent_id), sub_pods);
				}
				PodVerseAction::SpotTableChanged(story_id, width_height) => {
					pod_bank.relink_parents(story_id, width_height);
				}
				PodVerseAction::Relink(story_id) => {
					pod_bank.relink(&story_id);
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
