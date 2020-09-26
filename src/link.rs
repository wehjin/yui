use std::sync::mpsc::{channel, Sender, sync_channel, SyncSender};
use std::thread;

pub trait Link<A> {
	fn send(&self, action: A);
}

pub struct SenderLink<A> {
	pub tx: Sender<A>
}

impl<A> Clone for SenderLink<A> {
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
	pub fn new<B: Send + 'static>(sender: Sender<B>, f: impl Fn(A) -> B + Send + 'static) -> Self {
		let (tx, rx) = channel();
		thread::Builder::new().name("SenderLink::new".to_string()).spawn(move || {
			for a in rx {
				let b = f(a);
				sender.send(b).expect("send B");
			}
		}).expect("spawn");
		SenderLink { tx }
	}
	pub fn new_f(f: impl Fn(A) + Send + 'static) -> Self {
		let f = Box::new(f);
		let (tx, rx) = channel();
		thread::Builder::new().name("SenderLink::new_f".to_string()).spawn(move || {
			for a in rx { f(a) }
		}).expect("spawn");
		SenderLink { tx }
	}
	pub fn ignore() -> Self { Self::new_f(|_| {}) }
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

pub struct SyncLink<A> {
	tx: SyncSender<A>
}

impl<A> Clone for SyncLink<A> {
	fn clone(&self) -> Self {
		SyncLink { tx: self.tx.clone() }
	}
}

impl<A> Link<A> for SyncLink<A> {
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
	pub fn new(f: impl Fn(A) + Sync + Send + 'static) -> Self {
		let f = Box::new(f);
		let (tx, rx) = sync_channel(100);
		thread::Builder::new().name("SyncLink new".to_string()).spawn(move || {
			for a in rx { f(a) }
		}).expect("spawn");
		SyncLink { tx }
	}
	pub fn ignore() -> Self {
		Self::new(|_| {})
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
