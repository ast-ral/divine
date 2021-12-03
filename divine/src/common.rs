pub const CORRUPTION: &[u16] = &[
	161,
	162,
	193,
	164,
	195,
	166,
	167,
	168,
	169,
	170,
];

pub fn rand_under(rng_out: f64, under: usize) -> usize {
	(rng_out * under as f64) as usize
}
