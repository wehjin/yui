use std::error::Error;
use std::sync::mpsc::sync_channel;
use std::thread;

use crate::{Link, Projector, Spark};
use crate::app::yard_stack::YardStack;
use crate::yard::YardPublisherSource;

pub use self::edge::*;

pub(crate) mod yard_stack;

mod edge;

pub fn run<S>(spark: S, report_link: Option<Link<S::Report>>) -> Result<(), Box<dyn Error>>
	where S: Spark + Sync + Send + 'static
{
	let (yard_tx, yard_rx) = sync_channel(64);
	let on_close = {
		let yard_tx = yard_tx.clone();
		Link::new(move |_| {
			yard_tx.send(None).unwrap()
		})
	};
	let yard_stack = YardStack {};
	let stack_story = yard_stack.spark(None, Some(on_close));
	stack_story.link().send({
		let edge = Edge::new(stack_story.link());
		let app_story = spark.spark(Some(edge), report_link);
		yard_stack::Action::Push(app_story.yard_publisher())
	});
	{
		let yard_tx = yard_tx.clone();
		thread::spawn(move || {
			let yards = stack_story.yard_publisher().subscribe();
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
