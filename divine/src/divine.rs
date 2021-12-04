use std::collections::HashMap;
use std::rc::Rc;

use crate::common::{CORRUPTION, rand_under};
use crate::random::RngState;

pub fn divine(mut random: impl FnMut() -> f64, mut target: impl FnMut() -> Vec<u16>) -> Vec<Vec<u16>> {
	let mut predicted = RngState::lock_random(&mut random);

	let (corruption_bound, base_len) = divine_corruption_bound_and_base_len(
		&mut predicted,
		&mut random,
		&mut target,
	);

	let fragment_count = divine_fragment_count(
		&mut predicted,
		&mut random,
		&mut target,
		corruption_bound,
		base_len,
	);

	let fragments= divine_fragments(
		&mut predicted,
		&mut random,
		&mut target,
		corruption_bound,
		base_len,
		fragment_count,
	);

	fragments
}

/// Determines the corruption bound and base length. The corruption bound is 1
/// more than the maximum number of corruption characters in an output from a
/// fragment script. Fragment scripts internally use
/// `Math.floor(Math.random() * corruption_bound)` to determine the number of
/// corruption characters to place. The base length is just the length of the
/// base the fragment script is using.
fn divine_corruption_bound_and_base_len(
	predicted: &mut RngState,
	mut random: impl FnMut() -> f64,
	mut target: impl FnMut() -> Vec<u16>,
) -> (usize, usize) {
	// manipulate RNG to generate maximum corruption
	while predicted.next_random() < 0.99 {
		spin_random(&mut random, 1);
	}

	// call target and record the length of the output, which will be useful later
	let base_len = target().len();

	// count number of Math.random() calls the target made
	let mut count = 0;
	let barrier = random();

	while predicted.next_random() != barrier {
		count += 1;
	}

	// In the fragment scripts, the first Math.random() call determines how much
	// corruption there will be. After that call, 2 calls are made for each
	// corruption character. At the end, an additional 2 calls are made to place
	// the fragment. The maximum corruption is 1 less than the corruption bound,
	// so (corruption_bound - 1) * 2 + 2 calls are made in total, not counting
	// the initial call to determine how much corruption there is. This is
	// algebraically equivalent to corruption_bound * 2, so we can simply divide
	// the number of Math.random() calls by 2 to get the corruption bound.
	let corruption_bound = count / 2;

	// We're gonna move a single corruption char around later to determine the
	// length of the fragments and if `corruption_bound == 1`, then no
	// corruption can ever be generated.
	if corruption_bound == 1 {
		panic!("corruption bound too low to determine fragment lengths, aborting");
	}

	(corruption_bound, base_len)
}

/// Determines the number of fragments in the internal array of the fragment
/// script. This is probably the most complicated part of the algorithm.
/// It essentially works by considering a set of candidate fragment counts,
/// and eliminating those that are inconsistent with the actual data.
fn divine_fragment_count(
	predicted: &mut RngState,
	mut random: impl FnMut() -> f64,
	mut target: impl FnMut() -> Vec<u16>,
	corruption_bound: usize,
	base_len: usize,
) -> usize {
	// candidate fragment counts
	let candidates = 1 ..= 15;

	let mut consistency_maps: HashMap<_, _> = candidates
		.clone()
		.map(|candidate| (candidate, HashMap::new()))
		.collect();

	loop {
		// manipulate RNG to generate no corruption
		if predicted.next_random() >= 1.0 / corruption_bound as f64 {
			spin_random(&mut random, 1);
			continue;
		}

		// record the Math.random() call that determines which fragment is placed
		let fragment_selector = predicted.next_random();

		// Manipulate RNG to place the fragment at the start of the output.
		// The start of the fragment is essentially determined by
		// `Math.floor(Math.random() * (base.length - fragment.length))`
		// so getting a number under `1 / (base.length - 1)` is sufficient to
		// always place the fragment at the beginning. All fragments are at
		// least length 1, so we can use `base.length - 1` as our denominator
		// instead of just `base.length`.
		if predicted.next_random() >= 1.0 / (base_len - 1) as f64 {
			spin_random(&mut random, 3);
			continue;
		}

		let text = Rc::new(target());

		// only retain candidate fragment counts that are consistent with this output
		consistency_maps.retain(|&candidate, consistency_map| {
			let fragment_index = rand_under(fragment_selector, candidate);
			*consistency_map.entry(fragment_index).or_insert_with(|| text.clone()) == text
		});

		// If all consistent candidates are both complete and multiples of a
		// single candidate, that candidate is the fragment count.
		for &candidate in consistency_maps.keys() {
			let valid = consistency_maps
				.iter()
				.all(|(&k, v)| k % candidate == 0 && is_map_complete(v, k));

			if valid {
				return candidate;
			}
		}
	}
}

/// This function moves around a single corruption char to essentially binary
/// search the length of each fragment. Once the length is known, you can simply
/// extract the fragment by slicing the output of the fragment script.
fn divine_fragments(
	predicted: &mut RngState,
	mut random: impl FnMut() -> f64,
	mut target: impl FnMut() -> Vec<u16>,
	corruption_bound: usize,
	base_len: usize,
	fragment_count: usize,
) -> Vec<Vec<u16>> {
	let mut minmax_vec = vec![(0, base_len); fragment_count];
	let mut fragments = vec![Vec::new(); fragment_count];

	while !minmax_vec.iter().all(|k| k.0 == k.1) {
		// manipulate RNG to always generate a single corruption

		let min_bound = 1.0 / corruption_bound as f64;
		let max_bound = 2.0 / corruption_bound as f64;

		if !(min_bound .. max_bound).contains(&predicted.next_random()) {
			spin_random(&mut random, 1);
			continue;
		}

		// record where that corruption character will be placed
		let placement = rand_under(predicted.next_random(), base_len);

		// this determines which corruption char is used, we don't need to record it
		predicted.next_random();

		// record which fragment is put in the output
		let fragment_index = rand_under(predicted.next_random(), fragment_count);

		// these checks skip calling the target script if it would tell us no new information

		let min_bound = minmax_vec[fragment_index].0;
		let max_bound = minmax_vec[fragment_index].1;

		if !(min_bound .. max_bound).contains(&placement) {
			spin_random(&mut random, 4);
			continue;
		}

		// Make sure the fragment is at the start of the output, see comment in
		// `divine_fragment_count` for an explanation.
		if predicted.next_random() >= 1.0 / (base_len - 1) as f64 {
			spin_random(&mut random, 5);
			continue;
		}

		let text = target();

		// If the corruption char is in the text, it means that the fragment was
		// unable to cover it, and therefore the length of the fragment must be
		// less than the index where the corruption char was placed. If the
		// corruption char is *not* in the text, that means it was covered by
		// the fragment, and therefore the fragment has a length at least 1
		// greater than the index at which the corruption char was placed.
		if contains_corruption(&text) {
			minmax_vec[fragment_index].1 = placement;
		} else {
			minmax_vec[fragment_index].0 = placement + 1;
		}

		// If the minimum and maximum lengths of the fragment are equal, we know
		// the length for certain. We add it to the vector of outputs in this case.
		if minmax_vec[fragment_index].0 == minmax_vec[fragment_index].1 {
			fragments[fragment_index] = text[0 .. minmax_vec[fragment_index].0].to_owned();
		}
	}

	fragments
}

fn spin_random(mut random: impl FnMut() -> f64, count: usize) {
	for _ in 0 .. count {
		random();
	}
}

fn is_map_complete(map: &HashMap<usize, Rc<Vec<u16>>>, candidate: usize) -> bool {
	(0 .. candidate).all(|k| map.contains_key(&k))
}

fn contains_corruption(slice: &[u16]) -> bool {
	for elem in slice {
		if CORRUPTION.contains(elem) {
			return true;
		}
	}

	false
}
