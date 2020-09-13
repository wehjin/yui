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
	pub fn callback<Ctx>(&self, into_action: impl Fn(Ctx) -> A + Send) -> impl Fn(Ctx) {
		let tx = self.tx.to_owned();
		move |ctx: Ctx| {
			let action = into_action(ctx);
			(*tx)(action);
		}
	}
	pub fn new(tx: impl Fn(A) + 'static + Send + Sync) -> Self { SyncLink { tx: Arc::new(tx) } }
}
