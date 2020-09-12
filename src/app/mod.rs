use std::error::Error;
use std::sync::Arc;
use std::sync::mpsc::sync_channel;
use std::thread;

use crate::{Link, Projector, story, Story};
use crate::app::pub_stack::PubStack;
use crate::prelude::*;
use crate::yard::YardPublisher;

pub use self::edge::*;

pub(crate) mod pub_stack;

mod edge;

pub fn run<S>(spark: S, report_link: Option<Link<S::Report>>) -> Result<(), Box<dyn Error>>
	where S: Spark + Sync + Send + 'static
{
	let (yard_tx, yard_rx) = sync_channel(64);
	let on_close: Link<()> = {
		let yard_tx = yard_tx.clone();
		Link::new(move |_| yard_tx.send(None).unwrap())
	};
	let stack_story: Story<PubStack> = story::spark(PubStack {}, None, Some(on_close));
	stack_story.link().send({
		let app_story: Story<S> = story::spark(spark, Some(Edge::new(stack_story.link())), report_link);
		pub_stack::Action::Push(Arc::new(app_story))
	});
	{
		let yard_tx = yard_tx.clone();
		let yards = stack_story.subscribe().unwrap();
		thread::spawn(move || {
			for yard in yards {
				yard_tx.send(Some(yard)).unwrap();
			}
		});
	}
	Projector::project_yards(yard_rx)
}
