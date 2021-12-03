pub struct RngState {
	state: (u64, u64),
	cache_len: usize,
	cache: [f64; 64],
}

impl RngState {
	pub fn from_state(s0: u64, s1: u64) -> Self {
		let mut out = Self {
			state: (s0, s1),
			cache_len: 0,
			cache: [0.0; 64],
		};

		out.next_random();

		out
	}

	pub fn lock_random(mut random: impl FnMut() -> f64) -> Self {
		let (mut s_back, mut s_forwards) = loop {
			if let Some(states) = try_lock_random(&mut random) {
				break states;
			}
		};

		loop {
			let (new_s_back, new_s_forwards) = (back(s_forwards, s_back), s_back);

			if random() != int_to_float(new_s_back) {
				break;
			}

			s_back = new_s_back;
			s_forwards = new_s_forwards;
		}

		for _ in 0 .. 64 {
			let (new_s_back, new_s_forwards) = (s_forwards, next(s_back, s_forwards));

			s_back = new_s_back;
			s_forwards = new_s_forwards;
		}

		let mut out = Self {
			state: (s_back, s_forwards),
			cache_len: 0,
			cache: [0.0; 64],
		};

		out.next_random();

		out
	}

	pub fn next_random(&mut self) -> f64 {
		if self.cache_len == 0 {
			self.refill_cache();
		}

		self.cache_len -= 1;
		self.cache[self.cache_len]
	}

	fn refill_cache(&mut self) {
		let (mut s_back, mut s_forwards) = self.state;
		self.cache_len = 64;

		for i in 0 .. 64 {
			self.cache[i] = int_to_float(s_back);

			let (new_s_back, new_s_forwards) = (s_forwards, next(s_back, s_forwards));

			s_back = new_s_back;
			s_forwards = new_s_forwards;
		}

		self.state = (s_back, s_forwards);
	}
}

fn try_lock_random(mut random: impl FnMut() -> f64) -> Option<(u64, u64)> {
	let [f3, f2, f1, f0] = [
		random(),
		random(),
		random(),
		random(),
	];

	let k0 = float_to_int(f0);
	let k1 = float_to_int(f1);
	let k2 = float_to_int(f2);
	let k3 = float_to_int(f3);

	let s0 = k0 << 12 | derive_unknown(k0, k1, k2);
	let s1 = k1 << 12 | derive_unknown(k1, k2, k3);
	let s2 = next(s0, s1);
	let s3 = next(s1, s2);

	if int_to_float(s0) != f0 {
		return None;
	}

	if int_to_float(s1) != f1 {
		return None;
	}

	if int_to_float(s2) != f2 {
		return None;
	}

	if int_to_float(s3) != f3 {
		return None;
	}

	Some((s0, s1))
}

fn float_to_int(float: f64) -> u64 {
	let bytes = (float + 1.0).to_le_bytes();
	let num = u64::from_le_bytes(bytes);

	num & 0x000f_ffff_ffff_ffff
}

fn int_to_float(int: u64) -> f64 {
	let bytes = ((int >> 12) | 0x3ff0_0000_0000_0000).to_le_bytes();
	let float = f64::from_le_bytes(bytes);

	float - 1.0
}

fn derive_unknown(k0: u64, k1: u64, k2: u64) -> u64 {
	let mut x = k2 ^ (k1 >> 26) ^ k1;

	x ^= x >> 17;
	x ^= x >> 34;

	((x ^ k0 ^ (k0 << 23)) >> 11) & 0x0fff
}

fn back(s2: u64, s1: u64) -> u64 {
	let mut x = s2 ^ s1 ^ (s1 >> 26);

	x ^= x >> 17;
	x ^= x >> 34;

	x ^= x << 23;
	x ^= x << 46;

	x
}

fn next(s0: u64, s1: u64) -> u64 {
	let mut x = s0;

	x ^= x << 23;
	x ^= x >> 17;

	s1 ^ (s1 >> 26) ^ x
}
