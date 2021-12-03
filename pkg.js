const {
	readFileSync: read_file_sync,
	writeFileSync: write_file_sync,
	existsSync: exists_sync,
	unlinkSync: unlink_sync,
} = require("fs")

const file = read_file_sync("./target/wasm32-unknown-unknown/release/wasm_object.wasm")
const data = [...file]
const hex = data.map(i => i.toString(16).padStart(2, "0"))

const FILE_SIZE = 50000
const HEX_SIZE = FILE_SIZE / 2

for (let i = 0;; i++) {
	if (exists_sync(`./as_hex/data_${i}.txt`)) {
		unlink_sync(`./as_hex/data_${i}.txt`)
	} else {
		break
	}
}

for (let i = 0; i * HEX_SIZE < hex.length; i++) {
	write_file_sync(`./as_hex/data_${i}.txt`, hex.slice(HEX_SIZE * i, HEX_SIZE * (i + 1)).join(""))
}
