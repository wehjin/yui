
use std::sync::mpsc::{channel, Sender, sync_channel, SyncSender};
use std::thread;

pub trait Link<A: Send> {
	fn send(&self, action: A);
}

pub trait Sendable: Clone + Send + 'static {
	fn send(self, sender_link: &SenderLink<Self>) { sender_link.send(self); }
	fn send2(self, sender: &Sender<Self>, msg: &str) { sender.send(self).expect(msg); }
	fn into_trigger(self, sender: &Sender<Self>) -> Trigger { trigger(self.clone(), sender) }
	fn into_trigger_link(self, sender_link: &SenderLink<Self>) -> Trigger {
		sender_link.map(move |_| self.clone())
	}
	fn to_send<B: Send + 'static>(self, link: &SenderLink<Self>) -> SenderLink<B> {
		let link = link.clone();
		SenderLink::wrap_sink(move |_: B| { link.send(self.clone()) })
	}
	fn to_sync<B: Send + 'static>(self, link: &SenderLink<Self>) -> SyncLink<B> {
		let link = link.clone();
		SyncLink::wrap_sink(move |_: B| { link.send(self.clone()) })
	}
}

#[derive(Debug)]
pub struct SenderLink<A: Send> {
	pub tx: Sender<A>,
}

impl<A: Send> Clone for SenderLink<A> {
	fn clone(&self) -> Self {
		SenderLink { tx: self.tx.clone() }
	}
}

impl<A: Send> Link<A> for SenderLink<A> {
	fn send(&self, action: A) {
		self.tx.send(action).expect("Send Action")
	}
}

impl<A: Send + 'static> SenderLink<A> {
	pub fn to_sync(&self) -> SyncLink<A> {
		SyncLink::from(self.clone())
	}
	pub fn wrap_sender<B: Send + 'static>(sender: Sender<B>, f: impl Fn(A) -> B + Send + 'static) -> Self {
		let (tx, rx) = channel();
		thread::Builder::new().name("SenderLink::new".to_string()).spawn(move || {
			for a in rx {
				let b = f(a);
				sender.send(b).expect("send B");
			}
		}).expect("spawn");
		SenderLink { tx }
	}
	pub fn wrap_sink(sink: impl Fn(A) + Send + 'static) -> Self {
		let sink_a = Box::new(sink);
		let (tx, rx) = channel();
		thread::Builder::new().name("SenderLink::new_f".to_string()).spawn(move || {
			for a in rx {
				sink_a(a)
			}
		}).expect("spawn");
		SenderLink { tx }
	}
	pub fn ignore() -> Self { Self::wrap_sink(|_| {}) }
	pub fn map<B: Send + 'static>(&self, f: impl Fn(B) -> A + Send + 'static) -> SenderLink<B> {
		let f = Box::new(f);
		let link = self.clone();
		let (tx, rx) = channel();
		thread::Builder::new().name("SenderLink::map".to_string()).spawn(move || {
			for b in rx {
				link.send(f(b))
			}
		}).expect("spawn");
		SenderLink { tx }
	}
	pub fn callback<B>(&self, f: impl Fn(B) -> A + Send) -> impl Fn(B) {
		let link = self.clone();
		move |b: B| link.send(f(b))
	}
}

pub type Trigger = SenderLink<()>;

pub fn trigger<F: Clone + Send + 'static>(value: F, sender: &Sender<F>) -> Trigger {
	SenderLink::wrap_sender(sender.clone(), move |_| value.clone())
}

#[derive(Debug)]
pub struct SyncLink<A> {
	tx: SyncSender<A>,
}

impl<A> Clone for SyncLink<A> {
	fn clone(&self) -> Self {
		SyncLink { tx: self.tx.clone() }
	}
}

impl<A: Send> Link<A> for SyncLink<A> {
	fn send(&self, action: A) {
		self.tx.send(action).expect("send action")
	}
}

impl<A: Send + 'static> From<SenderLink<A>> for SyncLink<A> {
	fn from(sender_link: SenderLink<A>) -> Self {
		let (tx, rx) = sync_channel(100);
		thread::Builder::new().name("SenderLink from SenderLink".to_string()).spawn(move || {
			for a in rx {
				sender_link.send(a)
			}
		}).expect("spawn");
		SyncLink { tx }
	}
}

impl<A: Send + 'static> SyncLink<A> {
	pub fn wrap_sink(f: impl Fn(A) + Send + 'static) -> Self {
		let sink_a = Box::new(f);
		let (tx, rx) = sync_channel(100);
		thread::Builder::new().name("SyncLink new".to_string()).spawn(move || {
			for a in rx {
				sink_a(a)
			}
		}).expect("spawn");
		SyncLink { tx }
	}
	pub fn ignore() -> Self {
		Self::wrap_sink(|_| {})
	}
	pub fn map<B: Send + 'static>(self, f: impl Fn(B) -> A + Send + 'static) -> SyncLink<B> {
		let f = Box::new(f);
		let link = self;
		let (tx, rx) = sync_channel(100);
		thread::Builder::new().name("SyncLink map".to_string()).spawn(move || {
			for b in rx {
				link.send(f(b))
			}
		}).expect("spawn");
		SyncLink { tx }
	}
	pub fn callback<B>(&self, f: impl Fn(B) -> A + Send) -> impl Fn(B) {
		let link = self.clone();
		move |b: B| link.send(f(b))
	}
}
