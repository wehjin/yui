use std::error::Error;
use std::sync::mpsc::channel;
use std::thread;

use crate::{Link, Projector, SenderLink, story, Story};
use crate::app::pub_stack::PubStack;
use crate::prelude::*;
use crate::yard::YardPublisher;

pub use self::edge::*;

pub(crate) mod pub_stack;

mod edge;

enum RefresherAction {
	Enable(SenderLink<()>),
	Refresh,
}

fn start_refresher() -> SenderLink<RefresherAction> {
	let (tx, rx) = channel();
	thread::spawn(move || {
		let mut refresh_link: Option<SenderLink<()>> = None;
		for msg in rx {
			match msg {
				RefresherAction::Enable(link) => refresh_link = Some(link),
				RefresherAction::Refresh => if let Some(ref link) = refresh_link {
					link.send(())
				},
			}
		}
	});
	SenderLink { tx }
}

pub fn run<S>(spark: S, report_link: Option<SenderLink<S::Report>>) -> Result<(), Box<dyn Error>>
	where S: Spark + Sync + Send + 'static
{
	let refresher = start_refresher();
	let (yard_tx, yard_rx) = channel();
	let on_close: SenderLink<()> = {
		let yard_tx = yard_tx.clone();
		SenderLink::new(yard_tx, |_| None)
	};
	let refresh_link = refresher.clone().map(|_| RefresherAction::Refresh);
	let stack_edge = Edge::new(SenderLink::ignore(), refresh_link.clone());
	let stack_story: Story<PubStack> = story::spark(PubStack {}, Some(stack_edge), Some(on_close));
	stack_story.link().send({
		let app_edge = Edge::new(stack_story.link(), refresh_link.clone());
		let app_story: Story<S> = story::spark(spark, Some(app_edge), report_link);
		pub_stack::Action::Push(app_story.connect())
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
	let enable_refresher = refresher.clone().map(|refresh_link| RefresherAction::Enable(refresh_link));
	Projector::project_yards(yard_rx, enable_refresher)
}
