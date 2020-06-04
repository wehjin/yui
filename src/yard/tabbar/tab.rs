pub trait Tab {
	fn uid(&self) -> i32;
	fn label(&self) -> &str;
}

impl Tab for (i32, &str) {
	fn uid(&self) -> i32 {
		let (uid, _) = self;
		*uid
	}

	fn label(&self) -> &str {
		let (_, label) = self;
		*label
	}
}
