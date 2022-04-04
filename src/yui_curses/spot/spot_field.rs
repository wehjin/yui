#[derive(Debug, Clone)]
pub struct SpotField<T: Clone> {
	pub value: T,
	pub z: i32,
}

impl<T: Clone> SpotField<T> {
	pub fn new(value: T) -> Self { SpotField { value, z: i32::MAX } }
	pub fn set_near_equal(&mut self, value: T, z: i32) {
		if z <= self.z {
			self.value = value;
			self.z = z;
		}
	}
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


