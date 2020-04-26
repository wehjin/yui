use std::error::Error;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, sync_channel};
use std::thread;

pub use fade::*;
pub use fill::*;
pub use label::*;
pub use textfield::*;

use crate::story::Story;
use crate::Teller;
use crate::yui::layout::LayoutContext;
use crate::yui::palette::FillColor;
use crate::yui::RenderContext;

mod fade;
mod fill;
mod label;
mod textfield;


pub type ArcYard = Arc<dyn Yard + Sync + Send>;

pub trait Yard {
	fn id(&self) -> i32;
	fn update(&self, option: YardOption);
	fn layout(&self, ctx: &mut LayoutContext) -> usize;
	fn render(&self, ctx: &dyn RenderContext);
}

pub enum YardOption {
	FillColor(FillColor)
}

pub trait Publisher {
	fn yards(&self) -> Result<Receiver<ArcYard>, Box<dyn Error>>;
}

impl<T: Teller + 'static> Publisher for Story<T> {
	fn yards(&self) -> Result<Receiver<ArcYard>, Box<dyn Error>> {
		let visions = self.visions(rand::random())?;
		let (tx_yard, rx_yard) = sync_channel::<ArcYard>(64);
		let link = self.link();
		thread::spawn(move || {
			let mut done = false;
			while !done {
				let vision = visions.recv().unwrap();
				if let Some(yard) = T::yard(&vision, &link) {
					if let Err(_) = tx_yard.send(yard) {
						done = true;
					}
				};
			}
		});
		Ok(rx_yard)
	}
}
