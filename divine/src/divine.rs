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

fn divine_corruption_bound_and_base_len(
	predicted: &mut RngState,
	mut random: impl FnMut() -> f64,
	mut target: impl FnMut() -> Vec<u16>,
) -> (usize, usize) {
	while predicted.next_random() < 0.99 {
		spin_random(&mut random, 1);
	}

	let base_len = target().len();

	let mut count = 0;
	let barrier = random();

	while predicted.next_random() != barrier {
		count += 1;
	}

	let corruption_bound = count / 2;

	if corruption_bound == 1 {
		panic!("corruption bound too low to determine fragment lengths, aborting");
	}

	(corruption_bound, base_len)
}

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
		if predicted.next_random() >= 1.0 / corruption_bound as f64 {
			spin_random(&mut random, 1);
			continue;
		}

		let fragment_selector = predicted.next_random();

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

		// if a candidate and all its multiples are both consistent and complete
		// that candidate is the fragment count
		for candidate in candidates.clone() {
			let valid = (candidate ..)
				.step_by(candidate)
				.take_while(|multiple| candidates.contains(multiple))
				.all(|multiple| {
					consistency_maps.get(&multiple).map_or(false, |map| {
						is_map_complete(map, multiple)
					})
				});

			if valid {
				return candidate;
			}
		}
	}
}

fn divine_fragments(
	predicted: &mut RngState,
	mut random: impl FnMut() -> f64,
	mut target: impl FnMut() -> Vec<u16>,
	corruption_bound: usize,
	base_len: usize,
	fragment_count: usize,
) -> Vec<Vec<u16>> {
	let mut minmax_vec: Vec<_> = (0 .. fragment_count).map(|_| (0, base_len)).collect();
	let mut fragments: Vec<_> = (0 .. fragment_count).map(|_| Vec::new()).collect();

	while !minmax_vec.iter().all(|k| k.0 == k.1) {
		let min_bound = 1.0 / corruption_bound as f64;
		let max_bound = 2.0 / corruption_bound as f64;

		if !(min_bound .. max_bound).contains(&predicted.next_random()) {
			spin_random(&mut random, 1);
			continue;
		}

		let placement = rand_under(predicted.next_random(), base_len);

		predicted.next_random();

		let fragment_index = rand_under(predicted.next_random(), fragment_count);

		let min_bound = minmax_vec[fragment_index].0;
		let max_bound = minmax_vec[fragment_index].1;

		if !(min_bound .. max_bound).contains(&placement) {
			spin_random(&mut random, 4);
			continue;
		}

		if predicted.next_random() >= 1.0 / (base_len - 1) as f64 {
			spin_random(&mut random, 5);
			continue;
		}

		let text = target();

		if contains_corruption(&text) {
			minmax_vec[fragment_index].1 = placement;
		} else {
			minmax_vec[fragment_index].0 = placement + 1;
		}

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
