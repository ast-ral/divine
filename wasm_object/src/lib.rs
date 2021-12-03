use divine::divine::divine;
use interface_rs::{RawVec, VecExt, random};

#[link(wasm_import_module = "target")]
extern {
	fn target_callback() -> *mut RawVec<u16>;
}

#[no_mangle]
fn main() -> Box<RawVec<RawVec<u16>>> {
	let callback = || {
		unsafe {
			Box::from_raw(target_callback()).to_real_vec()
		}
	};

	let out = divine(random, callback);
	let out: Vec<_> = out.into_iter().map(|v| v.to_raw_vec()).collect();
	let out = Box::new(out.to_raw_vec());

	out
}
