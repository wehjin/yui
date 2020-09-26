use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::channel;
use std::thread;

use crate::{ArcYard, Link, RenderContext};
use crate::layout::LayoutContext;
use crate::yard::{Yard, YardOption, YardPublisher};

mod publisher;

pub fn publisher(publisher: &impl YardPublisher, refresh: impl Link<()> + Send + 'static) -> ArcYard {
	let id = rand::random();
	let yard_lock: Arc<RwLock<(u64, Option<ArcYard>)>> = Arc::new(RwLock::new((0, None)));
	let thread_yard_lock = yard_lock.clone();
	let yards = publisher.subscribe().expect("subscribe publisher");
	let (emit_yard_found, recv_yard_found) = channel();
	thread::spawn(move || {
		let mut yard_found = false;
		for yard in yards {
			{
				let mut write = thread_yard_lock.write().expect("write thread_yard_lock");
				let count = write.deref().0;
				*write = (count + 1, Some(yard));
			}
			if !yard_found {
				yard_found = true;
				emit_yard_found.send(()).expect("send () to emit_yard_found");
			}
			refresh.send(());
		}
	});
	recv_yard_found.recv().expect("receive yard_found");
	Arc::new(PublisherYard { id, yard_lock, layout_yard_num_lock: Arc::new(RwLock::new(0)) })
}

struct PublisherYard {
	id: i32,
	yard_lock: Arc<RwLock<(u64, Option<ArcYard>)>>,
	layout_yard_num_lock: Arc<RwLock<u64>>,
}

impl Yard for PublisherYard {
	fn render(&self, ctx: &dyn RenderContext) {
		let (yard_num, some_yard) = self.yard_lock.read().expect("read yard_lock").deref().clone();
		let layout_yard_num = self.layout_yard_num_lock.read().expect("read layout_yard_num_lock").deref().clone();
		if layout_yard_num == yard_num {
			match some_yard {
				None => {}
				Some(yard) => yard.render(ctx),
			}
		}
	}
	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (yard_num, bounds_id) = {
			let (yard_num, some_yard) = self.yard_lock.read().expect("read yard_lock").deref().clone();
			(
				yard_num,
				match some_yard {
					None => ctx.edge_bounds().0,
					Some(yard) => yard.layout(ctx),
				}
			)
		};
		{
			let mut layout_yard_num = self.layout_yard_num_lock.write().expect("write layout_yard_num_lock");
			*layout_yard_num = yard_num;
		}
		bounds_id
	}
	fn update(&self, _option: YardOption) {}
	fn id(&self) -> i32 { self.id }
}
