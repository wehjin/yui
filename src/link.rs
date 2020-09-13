use std::sync::Arc;

pub trait Link<A> {
	fn send(&self, action: A);
}

pub struct SyncLink<A> {
	tx: Arc<dyn Fn(A) + Send + Sync>,
}

impl<A> Link<A> for SyncLink<A> {
	fn send(&self, action: A) {
		(self.tx)(action);
	}
}

impl<A> Clone for SyncLink<A> {
	fn clone(&self) -> Self {
		SyncLink { tx: self.tx.clone() }
	}
}

impl<A: Send> SyncLink<A> {
	pub fn callback<B>(&self, f: impl Fn(B) -> A + Send) -> impl Fn(B) {
		let link = self.clone();
		move |b: B| link.send(f(b))
	}
	pub fn new(tx: impl Fn(A) + 'static + Send + Sync) -> Self { SyncLink { tx: Arc::new(tx) } }
}
