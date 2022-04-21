use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::channel;
use std::thread;

use crate::{ArcYard, Bounds, DrawPad, Link};
use crate::layout::LayoutContext;
use crate::yard::{Yard, YardPublisher};

mod publisher;

pub fn publisher(publisher: &impl YardPublisher, refresh: impl Link<()> + Send + 'static) -> ArcYard {
	let id = rand::random();
	let yard_lock: Arc<RwLock<(u64, Option<ArcYard>)>> = Arc::new(RwLock::new((0, None)));
	let thread_yard_lock = yard_lock.clone();
	let yards = publisher.subscribe().expect("subscribe publisher");
	let (emit_yard_found, recv_yard_found) = channel();
	thread::Builder::new().name("publisher".to_string()).spawn(move || {
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
	}).expect("spawn");
	recv_yard_found.recv().expect("receive yard_found");
	Arc::new(PublisherYard { id, yard_lock, layout_yard_num_lock: Arc::new(RwLock::new(0)) })
}

struct PublisherYard {
	id: i32,
	yard_lock: Arc<RwLock<(u64, Option<ArcYard>)>>,
	layout_yard_num_lock: Arc<RwLock<u64>>,
}

impl Yard for PublisherYard {
	fn id(&self) -> i32 { self.id }
	fn type_desc(&self) -> &'static str { "Publisher" }
	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (yard_num, bounds_index) = {
			let (yard_num, some_yard) = self.yard_lock.read().expect("read yard_lock").deref().clone();
			let yard_bounds_index = match some_yard {
				None => ctx.edge_bounds().0,
				Some(yard) => yard.layout(ctx),
			};
			(yard_num, yard_bounds_index)
		};
		{
			let mut layout_yard_num = self.layout_yard_num_lock.write().expect("write layout_yard_num_lock");
			*layout_yard_num = yard_num;
		}
		ctx.set_yard_bounds(self.id, bounds_index);
		bounds_index
	}
	fn render(&self, _bounds: &Bounds, _focus_id: i32, _pad: &mut dyn DrawPad) -> Option<Vec<(ArcYard, Option<i32>)>> {
		let (yard_num, some_yard) = self.yard_lock.read().expect("read yard_lock").deref().clone();
		let layout_yard_num = self.layout_yard_num_lock.read().expect("read layout_yard_num_lock").deref().clone();
		if layout_yard_num == yard_num {
			match some_yard {
				None => None,
				Some(yard) => Some(vec![(yard.clone(), None)]),
			}
		} else {
			None
		}
	}
}
