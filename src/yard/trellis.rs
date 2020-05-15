use crate::{ArcYard, Cling, Confine, Pack};

pub fn trellis(height: i32, gap: i32, strands: Vec<ArcYard>) -> ArcYard {
	let tip = &strands[0];
	let tail = &strands[1..];
	tail.iter().fold(
		tip.clone(),
		|total, next| {
			let confined = next.clone().confine_height(height, Cling::LeftBottom);
			total.pack_bottom(height + gap, confined)
		},
	)
}
