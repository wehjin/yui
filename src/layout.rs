use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use crate::{ArcYard, Bounds, Focus, SenderLink, Trigger};
use crate::bounds::BoundsHold;
use crate::pod_verse::tree::PodBranch;
use crate::story_id::StoryId;
use crate::yui::layout::ActiveFocus;

#[derive(Clone)]
pub struct LayoutState {
	pub max_x: i32,
	pub max_y: i32,
	pub start_index: usize,
	pub bounds_hold: Rc<RefCell<BoundsHold>>,
	pub active_focus: ActiveFocus,
	pub dependencies: HashSet<(i32, StoryId)>,
}

impl LayoutState {
	pub fn to_branches(&self) -> HashSet<PodBranch> {
		self.dependencies.iter().map(|(yard_id, sub_story_id)| {
			let bounds_hold = self.bounds_hold.borrow();
			let bounds = if let Some(bounds) = bounds_hold.yard_bounds(*yard_id) {
				bounds.clone()
			} else {
				Bounds::new(0, 0)
			};
			PodBranch { story_id: *sub_story_id, bounds }
		}).collect::<HashSet<_>>()
	}
}


pub fn run(height: i32, width: i32, yard: &ArcYard, refresh_trigger: &Trigger, prev_focus: &ActiveFocus) -> LayoutState {
	trace!("Layout width: {}, height: {}", width, height);
	let (start_index, bounds) = BoundsHold::init(width, height);
	{
		trace!("Starting BoundsHold: {:?}", bounds.borrow());
	}
	let mut layout_ctx = LayoutContext::new(start_index, bounds.clone(), refresh_trigger.clone());
	yard.layout(&mut layout_ctx);
	let active_focus = layout_ctx.pop_active_focus(prev_focus);
	{
		trace!("Ending BoundsHold: {:?}", bounds.borrow());
	}
	let dependencies = layout_ctx.dependencies.borrow();
	LayoutState { max_x: width, max_y: height, start_index, bounds_hold: bounds, active_focus, dependencies: dependencies.clone() }
}

#[derive(Clone)]
pub struct LayoutContext {
	current_index: usize,
	bounds_hold: Rc<RefCell<BoundsHold>>,
	focus_vec: Rc<RefCell<Vec<Rc<Focus>>>>,
	focus_max: i32,
	refresh_link: SenderLink<()>,
	dependencies: Rc<RefCell<HashSet<(i32, StoryId)>>>,
}

impl LayoutContext {
	pub fn add_dependency(&mut self, yard_id: i32, story_id: StoryId) {
		trace!("Add dependency on {:?} to yard:{:?}", story_id, yard_id);
		(*self.dependencies).borrow_mut().insert((yard_id, story_id));
		trace!("Dependencies after add {:?}", self.dependencies);
	}
	pub fn refresh_fn(&self) -> SenderLink<()> { self.refresh_link.clone() }
	pub fn trapped_focus(&self) -> Option<Rc<Focus>> {
		self.focus_vec.borrow().last().map(|it| it.clone())
	}
	pub fn pop_active_focus(&mut self, past_active: &ActiveFocus) -> ActiveFocus {
		let available_foci = self.all_focus_in_range();
		to_active_focus(past_active, available_foci)
	}
	pub fn all_focus_in_range(&self) -> Vec<Rc<Focus>> {
		let all_focus = (*self.focus_vec).borrow().clone();
		all_focus.into_iter().filter(|it| it.is_in_range(self.focus_max)).collect()
	}

	pub fn current_index(&self) -> usize {
		self.current_index
	}

	pub fn edge_bounds(&self) -> (usize, Bounds) {
		let bounds_index = self.current_index;
		let bounds = self.bounds(bounds_index);
		(bounds_index, bounds)
	}

	pub fn bounds(&self, index: usize) -> Bounds {
		(*self.bounds_hold).borrow().bounds(index)
	}

	pub fn push_bounds(&mut self, bounds: &Bounds) -> usize {
		(*self.bounds_hold).borrow_mut().push_bounds(bounds)
	}

	pub fn set_yard_bounds(&mut self, yard_id: i32, bounds_index: usize) -> usize {
		(*self.bounds_hold).borrow_mut().insert_yard_bounds(yard_id, bounds_index);
		bounds_index
	}

	pub fn add_focus(&mut self, focus: Focus) {
		(*self.focus_vec).borrow_mut().push(Rc::new(focus));
	}

	pub fn set_focus_max(&mut self, focus_max: i32) {
		self.focus_max = focus_max
	}

	pub fn trap_foci(&self) -> Self {
		let mut new_context = self.to_owned();
		new_context.focus_vec = Rc::new(RefCell::new(Vec::new()));
		new_context
	}

	pub fn with_index(&self, index: usize) -> Self {
		LayoutContext { current_index: index, ..self.to_owned() }
	}

	pub fn new(current_index: usize, bounds_hold: Rc<RefCell<BoundsHold>>, refresh_link: SenderLink<()>) -> Self {
		LayoutContext {
			current_index,
			bounds_hold,
			focus_vec: Rc::new(RefCell::new(Vec::new())),
			focus_max: i32::max_value(),
			refresh_link,
			dependencies: Rc::new(RefCell::new(HashSet::new())),
		}
	}
}


pub fn to_active_focus(past_active: &ActiveFocus, available_foci: Vec<Rc<Focus>>) -> ActiveFocus {
	let (focus, peers) =
		if let ActiveFocus { focus: Some(past_focus), .. } = past_active {
			let (mut continuity_foci, new_foci): (Vec<Rc<Focus>>, Vec<Rc<Focus>>) = available_foci.into_iter().partition(|it| it.yard_id == past_focus.yard_id);
			if continuity_foci.is_empty() {
				pick_priority_focus(new_foci)
			} else {
				let focus = continuity_foci.remove(0);
				(Some(focus), new_foci)
			}
		} else {
			pick_priority_focus(available_foci)
		};
	ActiveFocus { focus, peers }
}

fn pick_priority_focus(mut candidates: Vec<Rc<Focus>>) -> (Option<Rc<Focus>>, Vec<Rc<Focus>>) {
	let (_, max_priority_index) = candidates.iter().enumerate().fold(
		(0, None),
		|(max_priority, max_priority_index), (focus_index, focus)| {
			if max_priority_index.is_none() || focus.priority > max_priority {
				(focus.priority, Some(focus_index))
			} else {
				(max_priority, max_priority_index)
			}
		},
	);
	match max_priority_index {
		None => (None, candidates),
		Some(index) => {
			let focus = candidates.remove(index);
			(Some(focus), candidates)
		}
	}
}
