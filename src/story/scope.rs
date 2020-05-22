use std::collections::HashMap;
use std::sync::mpsc::SyncSender;

use crate::{Link, Flow, Story, Spark};
use crate::app::Edge;

pub(super) struct StoryScope<V, A> {
	vision: V,
	watchers: HashMap<i32, SyncSender<V>>,
	link: Link<A>,
	edge: Option<Edge>,
}

impl<V: Clone, A> StoryScope<V, A> {
	pub fn set_vision(&mut self, vision: V, announce: bool) {
		self.vision = vision;
		if announce {
			self.watchers.iter().for_each(|(_, it)| {
				it.send(self.vision.clone()).unwrap()
			});
		}
	}

	pub fn add_watcher(&mut self, id: i32, watcher: SyncSender<V>) {
		assert!(!self.watchers.contains_key(&id));
		self.watchers.insert(id, watcher.clone());
		watcher.send(self.vision.clone()).unwrap();
	}

	pub fn new(vision: V, link: Link<A>, ctx: Option<Edge>) -> Self {
		StoryScope { vision, watchers: HashMap::new(), link, edge: ctx }
	}
}

impl<V, A> Flow<V, A> for StoryScope<V, A> {
	fn state(&self) -> &V {
		&self.vision
	}

	fn link(&self) -> &Link<A> { &self.link }

	fn start_prequel<S>(&self, spark: S) -> Story<S>
		where S: Spark + Sync + Send + 'static
	{
		match &self.edge {
			None => panic!("No context"),
			Some(ctx) => ctx.start_dialog::<S>(spark),
		}
	}

	fn end_prequel(&self) {
		match &self.edge {
			None => panic!("No context"),
			Some(ctx) => ctx.end_dialog(),
		}
	}
}
