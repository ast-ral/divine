use std::alloc::{Layout, alloc, dealloc};
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::slice;

mod imports {
	#[link(wasm_import_module = "js")]
	extern {
		pub fn random() -> f64;
	}
}

pub fn random() -> f64 {
	unsafe {imports::random()}
}

#[repr(C)]
pub struct RawVec<T> {
	ptr: *mut T,
	len: usize,
	cap: usize,
}

impl<T> RawVec<T> {
	pub fn to_real_vec(self) -> Vec<T> {
		// SAFETY: the only (safe) way to get a RawVec is from a normal Vec and it consumes ownership
		unsafe {Vec::from_raw_parts(self.ptr, self.len, self.cap)}
	}
}

impl<T> Deref for RawVec<T> {
	type Target = [T];

	fn deref(&self) -> &[T] {
		unsafe {slice::from_raw_parts(self.ptr, self.len)}
	}
}

pub trait VecExt<T> {
	fn to_raw_vec(self) -> RawVec<T>;
}

impl<T> VecExt<T> for Vec<T> {
	fn to_raw_vec(self) -> RawVec<T> {
		let mut vec = ManuallyDrop::new(self);

		let ptr = vec.as_mut_ptr();
		let len = vec.len();
		let cap = vec.capacity();

		RawVec {ptr, len, cap}
	}
}

#[no_mangle]
fn alloc_raw_vec(cap: usize, elem_size: usize, elem_align: usize) -> *mut u8 {
	let layout = Layout::from_size_align(elem_size * cap, elem_align).unwrap();
	let memory = unsafe {alloc(layout)};

	if memory.is_null() {
		panic!("alloc returned null pointer");
	}

	memory
}

#[no_mangle]
unsafe fn dealloc_raw_vec(ptr: *mut u8, cap: usize, elem_size: usize, elem_align: usize) {
	let layout = Layout::from_size_align(elem_size * cap, elem_align).unwrap();

	dealloc(ptr, layout)
}

#[no_mangle]
fn alloc_raw_box(elem_size: usize, elem_align: usize) -> *mut u8 {
	let layout = Layout::from_size_align(elem_size, elem_align).unwrap();
	let memory = unsafe {alloc(layout)};

	if memory.is_null() {
		panic!("alloc returned null pointer");
	}

	memory
}

#[no_mangle]
unsafe fn dealloc_raw_box(ptr: *mut u8, elem_size: usize, elem_align: usize) {
	let layout = Layout::from_size_align(elem_size, elem_align).unwrap();

	dealloc(ptr, layout)
}
