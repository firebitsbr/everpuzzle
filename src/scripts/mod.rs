mod cursor;
pub use self::cursor::*;

mod grid;
pub use self::grid::*;

mod adaptors;
pub use self::adaptors::*;

mod component;
pub use self::component::*;

mod combo_highlight;
pub use self::combo_highlight::*;

/// helper to generate a state enum used for BlockState and GarbageState
///
/// implements simple and complex enum inner variables
macro_rules! block_state {
	(
	 $enum_name:ident,
	 {
		 $(
		 $simple_state:ident, $simple_lower:ident $(,)?
		 ),*
	 },
	 {
		 $(
		   $complex_state:ident, $complex_lower:ident {
				   $(
				 $element:ident: $ty:ty,
				 )*
		   } $(,)?
		   ),*
	 }
	 ) => {
		/// all states with different data
		#[derive(Debug, Copy, Clone)]
			pub enum $enum_name {
			$(
				  $simple_state,
				  )*

				$(
				  $complex_state {
					  counter: u32,
					  finished: bool,

					  $(
						$element: $ty,
						)*
				  },
				  )*
		}
		
		paste::item! {
			pub trait OptionState {
					$(
					fn [<is_ $simple_lower>](self) -> bool;
						  )*
						
						$(
						  fn [<is_ $complex_lower>](self) -> bool;
					  fn [<$complex_lower _started>](self) -> bool;
					  fn [<$complex_lower _finished>](self) -> bool;
					  )*
			}
			
			impl OptionState for Option<&$enum_name> {
				$(
				/// checks if the given state matches the function name state
				fn [<is_ $simple_lower>](self) -> bool {
						  self.filter(|state| match state {
										  $enum_name::$simple_state => true,
										  _ => false
										  }).is_some()
					  }
					  )*
					
					$(
					  /// checks if the given state matches the function name state
					  fn [<is_ $complex_lower>](self) -> bool {
						  self.filter(|state| match state {
							  $enum_name::$complex_state { .. } => true,
							  _ => false
										  }).is_some()
					  }
					  
					  /// checks if the given state has any variables, if so does the counter == 0
					  fn [<$complex_lower _started>](self) -> bool {
						  self.filter(|state| match state {
							  $enum_name::$complex_state { counter, .. } => *counter == 0,
							  _ => false
										  }).is_some()
					  }
					  
					  /// checks if the given state has any variables, if finished == true
					  fn [<$complex_lower _finished>](self) -> bool {
						  self.filter(|state| match state {
							  $enum_name::$complex_state { finished, .. } => *finished,
							  _ => false
										  }).is_some()
					  }
					  )*
			}
			
			pub trait OptionMutState {
				$(
					  fn [<to_ $simple_lower>](self);
					  )*
					
					$(
					  fn [<to_ $complex_lower>](self, $(
														$element: $ty,
														)*);
					  )*
				}
			
			impl OptionMutState for Option<&mut $enum_name> {
				$(
				/// simply converts the current state to the new state named in the function
				fn [<to_ $simple_lower>](self) {
						  self.map(|s| *s = $enum_name::$simple_state);
				}
					  )*
					
					$(
				/// converts the current state to the new state named in the function with its additional variables included in the function header
				fn [<to_ $complex_lower>](
											  self,
											  $(
												$element: $ty,
												)*
											  ) {
						  self.map(|s| *s = $enum_name::$complex_state {
						counter: 0,
						finished: false,
						$(
						  $element,
						  )*
									   });
				}
					  )*
			}
		}
		
		paste::item! {
			impl $enum_name {
				$(
					  /// checks if the given state matches the function name state
					  pub fn [<is_ $simple_lower>](self) -> bool {
					  match self {
						  $enum_name::$simple_state => true,
						  _ => false
					  }
				  }
					
					  /// simply converts the current state to the new state named in the function
					  pub fn [<to_ $simple_lower>](&mut self) {
						  *self = $enum_name::$simple_state;
					  }
					  )*
					
					$(
					  /// checks if the given state matches the function name state
					  pub fn [<is_ $complex_lower>](self) -> bool {
						  match self {
							  $enum_name::$complex_state { .. } => true,
							  _ => false
						  }
					  }

					  /// checks if the given state has any variables, if so does the counter == 0
					  pub fn [<$complex_lower _started>](self) -> bool {
						  match self {
							  $enum_name::$complex_state { counter, .. } => counter == 0,
							  _ => false
						  }
					  }

					  /// checks if the given state has any variables, if finished == true
					  pub fn [<$complex_lower _finished>](self) -> bool {
						  match self {
							  $enum_name::$complex_state { finished, .. } => finished,
							  _ => false
						  }
					  }

					  /// converts the current state to the new state named in the function with its additional variables included in the function header
					  pub fn [<to_ $complex_lower>](
												&mut self,
												$(
												  $element: $ty,
												  )*
												) {
						  *self = $enum_name::$complex_state {
							  counter: 0,
							  finished: false,
							  $(
								$element,
								)*
						  };
					  }
					  )*
			}
		}
	}
}

mod block;
pub use self::block::*;

mod garbage;
pub use self::garbage::*;
