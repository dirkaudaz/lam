.PHONY: build
build: build.wasm build.web
	cargo build

.PHONY: build.wasm
build.wasm:
	cargo build --target wasm32-wasi --package lam-rts-wasm

.PHONY: build.web
build.web:
	wasm-pack build \
		--dev \
		--target web \
		--no-typescript \
		./lib/lam-rts-web \
		-- --package lam-rts-web

.PHONY: docs
docs:
	cargo doc --target-dir ./docs --workspace --no-deps

.PHONY: test
test:
	cargo test

.PHONY: release
release: release.wasm release.web
	cargo build --release
	tar czf release.tar.gz -C ./target/release/ \
		$(shell find ./target/release -name "lam" -or -name "lam.exe" \
		        | sed 's@./target/release/@@g' )

.PHONY: release.wasm
release.wasm:
	cargo build --release --target wasm32-wasi --package lam-rts-wasm

.PHONY: release.web
release.web:
	wasm-pack build \
		--release \
		--target web \
		--no-typescript \
		./lib/lam-rts-web \
		-- --package lam-rts-web

.PHONY: install
install: release
	cargo install --path ./lib/lam-bin

.PHONY: setup
setup:
	cargo install wasm-pack
	rustup target add wasm32-wasi
	rustup target add wasm32-unknown-unknown

.PHONY: clean
clean:
	cargo clean

.PHONY: fmt
fmt:
	cargo fmt
