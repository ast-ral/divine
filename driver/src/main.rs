use std::cell::RefCell;

use divine::divine::divine;
use divine::random::RngState;
use divine::sim::sim_fragment_script;

fn main() {
	let base: Vec<_> = (0 .. 100).collect();
	let fragments: &[&[u16]] = &[
		&[100, 101, 102, 103, 104, 105, 106, 107],
		&[500, 501, 502, 503, 504, 505, 506],
		&[500, 501, 502, 503, 504],
		&[500, 501, 502, 503, 504, 505, 506],
		&[102, 103, 104, 105, 106, 107],
		&[400],
		&[887, 400],
	];

	for i in 0 .. 100 {
		let rng = RefCell::new(RngState::from_state(0x0123_4567_89ab_cdef, 1337 * i));
		let random = || rng.borrow_mut().next_random();

		let sim = || sim_fragment_script(&random, &base, 4, fragments);

		assert_eq!(divine(random, sim), fragments);
	}
}
