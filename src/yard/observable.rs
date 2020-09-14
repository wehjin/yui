use std::error::Error;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, sync_channel, SyncSender};
use std::thread;

use crate::{ArcYard, Fade};

pub trait YardPublisher {
	fn subscribe(&self) -> Result<Receiver<ArcYard>, Box<dyn Error>>;
}

impl YardPublisher for Arc<dyn YardPublisher> {
	fn subscribe(&self) -> Result<Receiver<ArcYard>, Box<dyn Error>> { self.deref().subscribe() }
}

pub enum YardControlMsg {
	On(SyncSender<ArcYard>),
	Off,
	Forward(ArcYard),
}

impl YardPublisher for SyncSender<YardControlMsg> {
	fn subscribe(&self) -> Result<Receiver<ArcYard>, Box<dyn Error>> {
		let (tx, rx) = sync_channel(100);
		self.send(YardControlMsg::On(tx)).unwrap();
		Ok(rx)
	}
}

pub fn overlay(rear: Arc<dyn YardPublisher>, fore: Arc<dyn YardPublisher>) -> Arc<dyn YardPublisher> {
	Arc::new(OverlayYardPublisher { rear, fore })
}

struct OverlayYardPublisher {
	rear: Arc<dyn YardPublisher>,
	fore: Arc<dyn YardPublisher>,
}

impl YardPublisher for OverlayYardPublisher {
	fn subscribe(&self) -> Result<Receiver<ArcYard>, Box<dyn Error>> {
		let (tx_yard, rx_yard) = sync_channel::<ArcYard>(64);
		let (tx_pos_yard, rx_pos_yard) = sync_channel::<(bool, ArcYard)>(64);
		thread::spawn(move || {
			let mut rear_yard: Option<ArcYard> = None;
			let mut fore_yard: Option<ArcYard> = None;
			for (is_fore, yard) in rx_pos_yard {
				let combo_yard = if is_fore {
					fore_yard = Some(yard.to_owned());
					if let Some(rear_yard) = &rear_yard {
						rear_yard.to_owned().fade((10, 10), yard.to_owned())
					} else {
						yard.to_owned()
					}
				} else {
					rear_yard = Some(yard.to_owned());
					if let Some(fore_yard) = &fore_yard {
						yard.fade((10, 10), fore_yard.to_owned())
					} else {
						yard.to_owned()
					}
				};
				tx_yard.send(combo_yard).unwrap();
			}
		});
		let rear_rx = self.rear.subscribe()?;
		let rear_tx_pos_yard = tx_pos_yard.to_owned();
		thread::spawn(move || {
			for yard in rear_rx {
				rear_tx_pos_yard.send((false, yard)).unwrap();
			}
		});
		let fore_rx = self.fore.subscribe()?;
		let fore_tx_pos_yard = tx_pos_yard.to_owned();
		thread::spawn(move || {
			for yard in fore_rx {
				fore_tx_pos_yard.send((true, yard)).unwrap();
			}
		});
		Ok(rx_yard)
	}
}
