use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::thread;

use crate::{ArcYard, RenderContext, Spark, Story};
use crate::layout::LayoutContext;
use crate::yard::{Yard, YardOption, YardPublisher, YardPublisherSource};

mod publisher;

pub fn publisher(publisher: &impl YardPublisher) -> ArcYard {
	let id = rand::random();
	let yard_lock: Arc<RwLock<Option<ArcYard>>> = Arc::new(RwLock::new(None));
	let refresh_lock: Arc<RwLock<Option<Arc<dyn Fn() + Sync + Send>>>> = Arc::new(RwLock::new(None));
	let thread_yard_lock = yard_lock.clone();
	let thread_refresh_lock = refresh_lock.clone();
	let yards = publisher.subscribe().unwrap();
	thread::spawn(move || {
		for yard in yards {
			{
				let mut write = thread_yard_lock.write().unwrap();
				*write = Some(yard)
			}
			{
				let read = thread_refresh_lock.read().unwrap();
				if let Some(refresh) = read.deref() { (*refresh)() }
			}
		}
	});
	Arc::new(PublisherYard { id, yard_lock, refresh_lock })
}

struct PublisherYard {
	id: i32,
	yard_lock: Arc<RwLock<Option<ArcYard>>>,
	refresh_lock: Arc<RwLock<Option<Arc<dyn Fn() + Sync + Send>>>>,
}

impl Yard for PublisherYard {
	fn render(&self, ctx: &dyn RenderContext) {
		let some_yard = self.yard_lock.read().unwrap();
		match *some_yard {
			None => {}
			Some(ref yard) => yard.render(ctx)
		}
	}
	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		{
			let mut write = self.refresh_lock.write().unwrap();
			*write = Some(ctx.refresh_fn().clone());
		}
		let some_yard_yard = self.yard_lock.read().unwrap();
		match *some_yard_yard {
			None => {
				let (bounds_id, _bounds) = ctx.edge_bounds();
				ctx.set_yard_bounds(self.id, bounds_id);
				bounds_id
			}
			Some(ref yard) => yard.layout(ctx),
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
