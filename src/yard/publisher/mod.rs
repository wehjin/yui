use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::channel;
use std::thread;

use crate::{ArcYard, RenderContext};
use crate::layout::LayoutContext;
use crate::yard::{Yard, YardOption, YardPublisher};

mod publisher;

pub fn publisher(publisher: &impl YardPublisher) -> ArcYard {
	let id = rand::random();
	let yard_lock: Arc<RwLock<(u64, Option<ArcYard>)>> = Arc::new(RwLock::new((0, None)));
	let refresh_lock: Arc<RwLock<Option<Arc<dyn Fn() + Sync + Send>>>> = Arc::new(RwLock::new(None));
	let thread_yard_lock = yard_lock.clone();
	let thread_refresh_lock = refresh_lock.clone();
	let yards = publisher.subscribe().unwrap();
	let (emit_yard_found, recv_yard_found) = channel();
	thread::spawn(move || {
		let mut yard_found = false;
		for yard in yards {
			{
				let mut write = thread_yard_lock.write().unwrap();
				let count = write.deref().0;
				*write = (count + 1, Some(yard));
			}
			if !yard_found {
				yard_found = true;
				emit_yard_found.send(()).unwrap();
			}
			{
				let read = thread_refresh_lock.read().unwrap();
				if let Some(refresh) = read.deref() { (*refresh)() }
			}
		}
	});
	recv_yard_found.recv().unwrap();
	Arc::new(PublisherYard { id, yard_lock, refresh_lock, layout_yard_num_lock: Arc::new(RwLock::new(0)) })
}

struct PublisherYard {
	id: i32,
	yard_lock: Arc<RwLock<(u64, Option<ArcYard>)>>,
	refresh_lock: Arc<RwLock<Option<Arc<dyn Fn() + Sync + Send>>>>,
	layout_yard_num_lock: Arc<RwLock<u64>>,
}

impl Yard for PublisherYard {
	fn render(&self, ctx: &dyn RenderContext) {
		let (yard_num, some_yard) = self.yard_lock.read().unwrap().deref().clone();
		let layout_yard_num = self.layout_yard_num_lock.read().unwrap().deref().clone();
		if layout_yard_num == yard_num {
			match some_yard {
				None => {}
				Some(yard) => yard.render(ctx),
			}
		}
	}
	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		{
			let mut write = self.refresh_lock.write().unwrap();
			*write = Some(ctx.refresh_fn().clone());
		}
		let (yard_num, bounds_id) = {
			let (yard_num, some_yard) = self.yard_lock.read().unwrap().deref().clone();
			(
				yard_num,
				match some_yard {
					None => ctx.edge_bounds().0,
					Some(yard) => yard.layout(ctx),
				}
			)
		};
		{
			let mut layout_yard_num = self.layout_yard_num_lock.write().unwrap();
			*layout_yard_num = yard_num;
		}
		bounds_id
	}
	fn update(&self, _option: YardOption) {}
	fn id(&self) -> i32 { self.id }
}
