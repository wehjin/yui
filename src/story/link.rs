use std::sync::Arc;

pub struct Link<A> {
	pub tx: Arc<dyn Fn(A) + Send + Sync>,
}

impl<A> Clone for Link<A> {
	fn clone(&self) -> Self {
		Link { tx: self.tx.clone() }
	}
}

impl<A: Send> Link<A> {
	pub fn callback<Ctx>(&self, into_action: impl Fn(Ctx) -> A + Send) -> impl Fn(Ctx) {
		let tx = self.tx.to_owned();
		move |ctx: Ctx| {
			let action = into_action(ctx);
			(*tx)(action);
		}
	}
	pub fn send(&self, action: A) {
		self.callback(|a| a)(action);
	}
}
