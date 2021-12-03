function (context, input) { // target: 0
	const script_owner = context.this_script.split(".")[0]
	const cli = !(context.is_scriptor || context.calling_script)

	if (context.caller == script_owner && cli && input) {
		if (input.wasm_clear) {
			#db.r({_id: "divine#"})
			return {ok: true, msg: "data cleared"}
		}

		if (input.wasm_upload) {
			const doc = #db.f({_id: "divine#"}).first()
			const hex = (doc ? doc.hex : "") + input.wasm_upload
			#db.us({_id: "divine#"}, {$set: {hex}})
			return {ok: true, msg: "data uploaded"}
		}
	}

	const start = Date.now()

	if (!input || !input.target || !input.target.call) {
		return "pass a fragment script to `Ntarget`"
	}

	const target = input.target.call

	function get_wasm_bytes() {
		const hex = #db.f({_id: "divine#"}).first().hex
		const bytes = []

		for (let i = 0; i < hex.length; i += 2) {
			bytes.push(parseInt(hex.slice(i, i + 2), 16))
		}

		return new Uint8Array(bytes)
	}

	function is_little_endian() {
		const u32_arr = new Uint32Array(1)
		u32_arr[0] = 0xdeadc0de

		const view = new DataView(u32_arr.buffer)

		if (view.getUint32(0, true) == 0xdeadc0de) {
			return true
		}

		if (view.getUint32(0, false) == 0xdeadc0de) {
			return false
		}

		throw new Error("unknown endianness?")
	}

	const little_endian = is_little_endian()

	const module = new WebAssembly.Module(get_wasm_bytes())

	const get_u16 = x => (new DataView(memory.buffer)).getUint16(x, little_endian)
	const get_u32 = x => (new DataView(memory.buffer)).getUint32(x, little_endian)
	const set_u16 = (x, y) => (new DataView(memory.buffer)).setUint16(x, y, little_endian)
	const set_u32 = (x, y) => (new DataView(memory.buffer)).setUint32(x, y, little_endian)

	function target_callback() {
		const text = target()

		const box = alloc_raw_box(12, 4)
		const vec = alloc_raw_vec(text.length, 2, 2)

		set_u32(box + 0, vec)
		set_u32(box + 4, text.length)
		set_u32(box + 8, text.length)

		for (let i = 0; i < text.length; i++) {
			set_u16(vec + 2 * i, text.charCodeAt(i))
		}

		return box
	}

	const divine = new WebAssembly.Instance(module, {js: {random: Math.random}, target: {target_callback}})

	const {
		memory,
		alloc_raw_box,
		dealloc_raw_box,
		alloc_raw_vec,
		dealloc_raw_vec,
		main,
	} = divine.exports

	const fragments_box = main()
	const fragments_ptr = get_u32(fragments_box + 0)
	const fragments_len = get_u32(fragments_box + 4)
	const fragments_cap = get_u32(fragments_box + 8)

	const fragments = []

	for (let i = 0; i < fragments_len; i++) {
		const nums = []

		const fragment_ptr = get_u32(fragments_ptr + 12 * i + 0)
		const fragment_len = get_u32(fragments_ptr + 12 * i + 4)
		const fragment_cap = get_u32(fragments_ptr + 12 * i + 8)

		for (let j = 0; j < fragment_len; j++) {
			nums.push(get_u16(fragment_ptr + 2 * j))
		}

		dealloc_raw_vec(fragment_ptr, fragment_len, fragment_cap, 2, 2)

		fragments.push(String.fromCharCode(...nums))
	}

	dealloc_raw_vec(fragments_ptr, fragments_cap, 12, 4)
	dealloc_raw_box(fragments_box, 12, 4)

	return {ok: true, fragments, time: Date.now() - start}
}
