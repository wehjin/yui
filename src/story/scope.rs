use std::collections::HashMap;
use std::sync::mpsc::Sender;

use crate::{Flow, Link, SenderLink, Spark, Story};
use crate::app::Edge;

pub(super) struct StoryScope<V, A, R> {
	vision: V,
	watchers: HashMap<i32, Sender<V>>,
	link: SenderLink<A>,
	edge: Option<Edge>,
	on_report: SenderLink<R>,
}

impl<V: Clone, A, R: Send + 'static> StoryScope<V, A, R> {
	pub fn set_vision(&mut self, vision: V, announce: bool) {
		self.vision = vision;
		if announce {
			self.watchers.iter().for_each(|(_, it)| {
				it.send(self.vision.clone()).expect("send vision to watcher")
			});
		}
	}

	pub fn add_watcher(&mut self, id: i32, watcher: Sender<V>) {
		assert!(!self.watchers.contains_key(&id));
		self.watchers.insert(id, watcher.clone());
		watcher.send(self.vision.clone()).expect("send vision to watcher");
	}

	pub fn new(vision: V, link: SenderLink<A>, edge: Option<Edge>, on_report: SenderLink<R>) -> Self {
		StoryScope { vision, watchers: HashMap::new(), link, on_report, edge }
	}
}

impl<S, A, R: Send + 'static> Flow<S, A, R> for StoryScope<S, A, R> {
	fn state(&self) -> &S { &self.vision }

	fn link(&self) -> &SenderLink<A> { &self.link }

	fn start_prequel<Sprk>(&self, spark: Sprk, on_report: SenderLink<Sprk::Report>) -> Story<Sprk>
		where Sprk: Spark + Send + 'static
	{
		match &self.edge {
			None => panic!("No edge in StoryScope"),
			Some(ctx) => ctx.start_dialog::<Sprk>(spark, on_report),
		}
	}

	fn end_prequel(&self) {
		match &self.edge {
			None => panic!("No edge in StoryScope"),
			Some(ctx) => ctx.end_dialog(),
		}
	}

	fn redraw(&self) {
		match &self.edge {
			None => panic!("No edge in StoryScope"),
			Some(ctx) => ctx.redraw(),
		}
	}


	fn report(&self, report: R) {
		self.on_report.send(report)
	}
}
