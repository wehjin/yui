use std::collections::HashMap;
use std::sync::mpsc::SyncSender;

use crate::UpdateContext;

pub(super) struct StoryScope<V> {
	vision: V,
	watchers: HashMap<i32, SyncSender<V>>,
}

impl<V: Clone> StoryScope<V> {
	pub fn set_vision(&mut self, vision: V) {
		self.vision = vision;
		self.watchers.iter().for_each(|(_, it)| {
			it.send(self.vision.clone()).unwrap()
		});
	}

	pub fn add_watcher(&mut self, id: i32, watcher: SyncSender<V>) {
		assert!(!self.watchers.contains_key(&id));
		self.watchers.insert(id, watcher.clone());
		watcher.send(self.vision.clone()).unwrap();
	}

	pub fn new(vision: V) -> Self {
		StoryScope { vision, watchers: HashMap::new() }
	}
}

impl<V> UpdateContext<V> for StoryScope<V> {
	fn vision(&self) -> &V {
		&self.vision
	}
}
