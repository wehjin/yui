use crate::{ArcYard, Cling, Confine, Pack};

pub fn trellis(height: i32, gap: i32, cling: Cling, strands: Vec<ArcYard>) -> ArcYard {
	//! Generate a yard that displays other yards in a vertical stack
	//! with uniform height and spacing.
	let tip = &strands[0];
	let tail = &strands[1..];
	tail.iter()
		.fold(
			tip.clone(),
			|total, next| {
				let confined = next.clone().confine_height(height, Cling::LeftBottom);
				total.pack_bottom(height + gap, confined)
			},
		)
		.confine_height((height + gap) * strands.len() as i32 - gap, cling)
}
