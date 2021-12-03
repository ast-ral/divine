use crate::common::{CORRUPTION, rand_under};

pub fn sim_fragment_script(
	mut random: impl FnMut() -> f64,
	base: &[u16],
	corruption_bound: usize,
	fragments: &[&[u16]],
) -> Vec<u16> {
	let mut out = base.to_owned();

	let num_corruption = rand_under(random(), corruption_bound);

	for _ in 0 .. num_corruption {
		let index = rand_under(random(), base.len());
		let corruption_char = CORRUPTION[rand_under(random(), CORRUPTION.len())];
		out[index] = corruption_char;
	}

	let fragment = fragments[rand_under(random(), fragments.len())];
	let fragment_start = rand_under(random(), base.len() - fragment.len());

	out[fragment_start ..][.. fragment.len()].copy_from_slice(fragment);

	out
}
