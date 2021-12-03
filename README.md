# divine

This project divines fragments from scripts in [hackmud](https://hackmud.com) via manipulation of the game's RNG.

## Building

You can build the project and convert the generated WebAssembly binary into hex files that can be uploaded with `cargo wasm && node ./pkg.js`.
The generated hex files will be put in the `./as_hex` directory.

## Uploading

The js file [here](./divine.js) can be directly uploaded into hackmud.
You can then upload the generated hex files one by one into the database by pasting them into the `{wasm_upload: ""}` argument.
