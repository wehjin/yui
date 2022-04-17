use std::error::Error;
use std::sync::mpsc::channel;
use std::thread;

use crate::{Link, ProjectorReport, run_console, SenderLink, story, Story};
use crate::app::pub_stack::PubStack;
use crate::prelude::*;
use crate::yard::YardPublisher;

pub use self::edge::*;

pub(crate) mod pub_stack;

mod edge;

pub fn run<S>(spark: S, report_link: Option<SenderLink<S::Report>>) -> Result<(), Box<dyn Error>>
	where S: Spark + Send + 'static
{
	let refresher = start_refresher();
	let (yard_tx, yard_rx) = channel();
	let on_close: SenderLink<()> = {
		let yard_tx = yard_tx.clone();
		SenderLink::wrap_sender(yard_tx, |_| None)
	};
	let refresh_link = refresher.clone().map(|_| RefresherAction::Refresh);
	let stack_edge = AppEdge::new(SenderLink::ignore(), refresh_link.clone());
	let stack_story: Story<PubStack> = story::spark(PubStack {}, Some(stack_edge), Some(on_close));
	stack_story.link().send({
		let app_edge = AppEdge::new(stack_story.link(), refresh_link.clone());
		let app_story: Story<S> = story::spark(spark, Some(app_edge), report_link);
		pub_stack::Action::Push(app_story.connect())
	});
	{
		let yard_tx = yard_tx.clone();
		let yards = stack_story.subscribe().expect("subscribe stack_story");
		thread::Builder::new().name("yui_curses:run".to_string()).spawn(move || {
			for yard in yards {
				yard_tx.send(Some(yard)).expect("send some yard");
			}
		}).expect("spawn");
	}
	let reports_link = refresher.clone().map(|report| {
		let ProjectorReport::Ready { refresh_trigger: refresh_link } = report;
		RefresherAction::Enable(refresh_link)
	});
	run_console(yard_rx, reports_link)
}

enum RefresherAction {
	Enable(SenderLink<()>),
	Refresh,
}

fn start_refresher() -> SenderLink<RefresherAction> {
	let (tx, rx) = channel();
	thread::Builder::new().name("start_refresher".to_string()).spawn(move || {
		let mut refresh_link: Option<SenderLink<()>> = None;
		for msg in rx {
			match msg {
				RefresherAction::Enable(link) => refresh_link = Some(link),
				RefresherAction::Refresh => if let Some(ref link) = refresh_link {
					link.send(())
				},
			}
		}
	}).expect("spawn");
	SenderLink { tx }
}
