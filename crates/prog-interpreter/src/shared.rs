use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{self, Debug};
use std::rc::Rc;

pub struct Shared<T: ?Sized> {
	value: Rc<RefCell<T>>
}

impl<T> Shared<T> {
	pub fn new(value: T) -> Self {
		Self {
			value: Rc::new(RefCell::new(value))
		}
	}

	pub fn unwrap_or_clone(self) -> T
	where
		T: Clone
	{
		let cell = Rc::unwrap_or_clone(self.value);
		cell.into_inner()
	}

	pub fn swap(&mut self, other: Self) -> Self { std::mem::replace(self, other) }
}

impl<T: ?Sized> Shared<T> {
	pub fn borrow(&'_ self) -> Ref<'_, T> { self.value.borrow() }

	pub fn borrow_mut(&'_ self) -> RefMut<'_, T> { self.value.borrow_mut() }
}

impl<T: ?Sized> Clone for Shared<T> {
	fn clone(&self) -> Self {
		Self {
			value: Rc::clone(&self.value)
		}
	}
}

impl<T: ?Sized + Debug> Debug for Shared<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = f.debug_tuple("Shared");
		s.field(&self.value.borrow());
		s.finish()
	}
}
