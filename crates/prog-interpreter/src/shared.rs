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

	pub fn unwrap_or_clone(this: Self) -> T
	where
		T: Clone
	{
		let cell = Rc::unwrap_or_clone(this.value);
		cell.into_inner()
	}

	pub fn swap(this: &mut Self, other: Self) -> Self { std::mem::replace(this, other) }

	pub fn ptr_eq(this: &Self, other: &Self) -> bool { Rc::ptr_eq(&this.value, &other.value) }
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

impl<T: ?Sized + PartialEq> PartialEq for Shared<T> {
	fn eq(&self, other: &Self) -> bool { *self.borrow() == *other.borrow() }
}

impl<T: ?Sized + Debug> Debug for Shared<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = f.debug_tuple("Shared");
		s.field(&self.value.borrow());
		s.finish()
	}
}

#[cfg(feature = "serde")]
impl<T: ?Sized + serde::Serialize> serde::Serialize for Shared<T> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer
	{
		serializer.serialize_newtype_struct("Shared", &*self.value)
	}
}
