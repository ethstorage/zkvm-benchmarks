build:
	mkdir -p target
	wasm-pack build --release --out-name guest_nonopt.wasm --out-dir pkg
	wasm-opt -Oz -o target/guest.wasm pkg/guest_nonopt.wasm

clean:
	rm -rf pkg target