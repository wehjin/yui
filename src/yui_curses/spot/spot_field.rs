#[derive(Debug, Clone)]
pub struct SpotField<T: Clone> {
	pub value: T,
	pub z: i32,
}

impl<T: Clone> SpotField<T> {
	pub fn new(value: T) -> Self { SpotField { value, z: i32::MAX } }
	pub fn expand_seam(&self, z: i32, depth: i32) -> Self {
		let adjustment = if self.z < z { -depth } else { 0 };
		SpotField {
			value: self.value.clone(),
			z: self.z + adjustment,
		}
	}
	pub fn insert_seam(&mut self, z: i32, field: &Self) {
		self.set_near_equal(field.value.clone(), field.z + z);
	}
	pub fn set_near_equal(&mut self, value: T, z: i32) {
		if z <= self.z {
			self.value = value;
			self.z = z;
		}
	}
	pub fn nearest_z(&self, z: i32) -> i32 { self.z.min(z) }
}


#[cfg(test)]
mod tests {
	use crate::spot::spot_field::SpotField;

	#[test]
	fn set_z() {
		let mut field = SpotField::new(0);
		field.set_near_equal(1, -1);
		field.set_near_equal(2, 0);
		assert_eq!(field.value, 1);
	}
}


