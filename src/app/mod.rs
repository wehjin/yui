use std::error::Error;
use std::sync::mpsc::sync_channel;
use std::thread;

use crate::{Link, Projector, Wheel};
use crate::app::yard_stack::YardStack;
use crate::yard::YardObservableSource;

pub use self::edge::*;

pub(crate) mod yard_stack;

mod edge;

pub fn run<W: Wheel>(report_link: Option<Link<W::Report>>) -> Result<(), Box<dyn Error>> {
	let (yard_tx, yard_rx) = sync_channel(64);
	let on_close = {
		let yard_tx = yard_tx.clone();
		Link::new(move |_| {
			yard_tx.send(None).unwrap()
		})
	};
	let stack_story = YardStack::launch(None, Some(on_close));
	stack_story.link().send({
		let edge = Edge::new(stack_story.link());
		let app_story = W::launch(Some(edge), report_link);
		yard_stack::Action::PushFront(app_story.yards())
	});
	{
		let yard_tx = yard_tx.clone();
		thread::spawn(move || {
			let yards = stack_story.yards().subscribe();
			match yards {
				Ok(yards) =>
					for yard in yards {
						yard_tx.send(Some(yard)).unwrap();
					},
				Err(_) => yard_tx.send(None).unwrap(),
			}
		});
	}
	Projector::project_yards(yard_rx)
}
