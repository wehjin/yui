use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::thread;

use crate::{ArcYard, RenderContext, Spark, Story};
use crate::layout::LayoutContext;
use crate::yard::{Yard, YardOption, YardPublisher, YardPublisherSource};

mod publisher;

pub fn story<Sprk: Spark + Sync + Send + 'static>(story: &Story<Sprk>) -> ArcYard {
	let id = rand::random();
	info!("yard::story id {}", id);
	let story = story.clone();
	let story_yard: Arc<RwLock<Option<ArcYard>>> = Arc::new(RwLock::new(None));
	let refresh: Arc<RwLock<Option<Arc<dyn Fn() + Sync + Send>>>> = Arc::new(RwLock::new(None));
	let thread_story_yard = story_yard.clone();
	let thread_refresh = refresh.clone();
	thread::spawn(move || {
		for yard in story.subscribe().unwrap() {
			{
				let mut write = thread_story_yard.write().unwrap();
				*write = Some(yard)
			}
			{
				let read = thread_refresh.read().unwrap();
				if let Some(refresh) = read.deref() { (*refresh)() }
			}
		}
	});
	Arc::new(StoryYard { id, story_yard, refresh })
}

struct StoryYard {
	id: i32,
	story_yard: Arc<RwLock<Option<ArcYard>>>,
	refresh: Arc<RwLock<Option<Arc<dyn Fn() + Sync + Send>>>>,
}

impl Yard for StoryYard {
	fn render(&self, ctx: &dyn RenderContext) {
		let story_yard = self.story_yard.read().unwrap();
		match *story_yard {
			None => {}
			Some(ref story_yard) => story_yard.render(ctx)
		}
	}
	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		{
			let mut write = self.refresh.write().unwrap();
			*write = Some(ctx.refresh_fn().clone());
		}
		let story_yard = self.story_yard.read().unwrap();
		match *story_yard {
			None => {
				let (bounds_id, _bounds) = ctx.edge_bounds();
				ctx.set_yard_bounds(self.id, bounds_id);
				bounds_id
			}
			Some(ref story_yard) => story_yard.layout(ctx),
		}
	}
	fn update(&self, _option: YardOption) {}
	fn id(&self) -> i32 { self.id }
}

impl<S> YardPublisherSource for Story<S> where S: Spark + Sync + Send + 'static {
	fn yard_publisher(&self) -> Arc<dyn YardPublisher> {
		Arc::new(self.clone())
	}
}
