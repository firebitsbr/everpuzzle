use crate::scripts::*;
use crate::helpers::*;

// block adaptors

pub trait OptionBlock {
	fn get_chain(self) -> Option<usize>;
	}

impl OptionBlock for Option<&Block> {
	fn get_chain(self) -> Option<usize> {
		match self {
			Some(b) => b.was_chainable,
			_ => None
		}
	}
}

trait OptionMutBlock {
	fn set_hframe(self, num: u32);
	fn set_chain(self, size: u32);
}

impl OptionMutBlock for Option<&mut Block> {
	fn set_hframe(self, num: u32) {
		match self {
			Some(b) => b.hframe = num,
			_ => {}
		}
	}
	
	fn set_chain(self, size: u32) {
		match self {
			Some(b) => b.was_chainable = Some(size as usize),
			_ => {}
		}
	}
}

// component adaptors

pub trait OptionMutComponent {
	fn to_chainable(self, chain: usize);
}

impl OptionMutComponent for Option<&mut Component> {
	fn to_chainable(self, chain: usize){
		match self {
			Some(c) => *c = Component::Chainable(chain),
			_ => {}
		}
		}
}
	
	