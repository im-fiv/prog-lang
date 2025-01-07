use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

#[derive(Debug)]
pub struct Shared<T: ?Sized> {
	value: Rc<RefCell<T>>
}

impl<T> Shared<T> {
	pub fn new(value: T) -> Self {
		Self {
			value: Rc::new(RefCell::new(value))
		}
	}

	pub fn borrow(&self) -> Ref<'_, T> { self.value.borrow() }

	pub fn borrow_mut(&self) -> RefMut<'_, T> { self.value.borrow_mut() }
}
