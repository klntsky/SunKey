web:
	@cargo build --target wasm32-unknown-unknown
	@cp target/wasm32-unknown-unknown/debug/trgls.wasm ./sunkey.wasm
