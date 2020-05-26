use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::SyncSender;

use crate::{Flow, Link, Spark, Story};
use crate::app::Edge;

pub(super) struct StoryScope<V, A, R> {
	vision: V,
	watchers: HashMap<i32, SyncSender<V>>,
	link: Link<A>,
	edge: Option<Edge>,
	on_report: Arc<dyn Fn(R) + Sync + Send + 'static>,
}

impl<V: Clone, A, R> StoryScope<V, A, R> {
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

	pub fn new(vision: V, link: Link<A>, ctx: Option<Edge>, on_report: impl Fn(R) + Sync + Send + 'static) -> Self {
		StoryScope { vision, watchers: HashMap::new(), link, on_report: Arc::new(on_report), edge: ctx }
	}
}

impl<S, A, R> Flow<S, A, R> for StoryScope<S, A, R> {
	fn state(&self) -> &S { &self.vision }

	fn link(&self) -> &Link<A> { &self.link }

	fn start_prequel<Sprk>(&self, spark: Sprk, on_report: impl Fn(Sprk::Report) + Sync + Send + 'static) -> Story<Sprk>
		where Sprk: Spark + Sync + Send + 'static
	{
		let report_link = Link::new(on_report);
		match &self.edge {
			None => panic!("No context"),
			Some(ctx) => ctx.start_dialog::<Sprk>(spark, report_link),
		}
	}

	fn end_prequel(&self) {
		match &self.edge {
			None => panic!("No context"),
			Some(ctx) => ctx.end_dialog(),
		}
	}

	fn report(&self, report: R) {
		(*self.on_report)(report)
	}
}
