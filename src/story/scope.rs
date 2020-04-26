use std::collections::HashMap;
use std::sync::mpsc::SyncSender;

use crate::{Link, UpdateContext};

pub(super) struct StoryScope<V, A> {
	vision: V,
	watchers: HashMap<i32, SyncSender<V>>,
	link: Link<A>,
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

	pub fn new(vision: V, link: Link<A>) -> Self {
		StoryScope {
			vision,
			watchers: HashMap::new(),
			link,
		}
	}
}

impl<V, A> UpdateContext<V, A> for StoryScope<V, A> {
	fn vision(&self) -> &V {
		&self.vision
	}

	fn link(&self) -> &Link<A> { &self.link }
}
