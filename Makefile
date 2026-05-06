.PHONY: all clean demo space radiogarden trailers

all: demo space radiogarden trailers
	@echo "All plugins built"

demo:
	cd plugins/demo && tinygo build -o ../../dist/demo.wasm -target=wasi -no-debug ./
	@echo "Built dist/demo.wasm ($$(ls -la dist/demo.wasm | awk '{print $$5}') bytes)"

radiogarden:
	cd plugins/radiogarden && cargo build --release --target wasm32-wasip1
	cp plugins/radiogarden/target/wasm32-wasip1/release/radiogarden_rust.wasm dist/radiogarden.wasm
	@echo "Built dist/radiogarden.wasm ($$(ls -la dist/radiogarden.wasm | awk '{print $$5}') bytes)"

trailers:
	cd plugins/trailers && tinygo build -o ../../dist/trailers.wasm -target=wasi -no-debug ./
	@echo "Built dist/trailers.wasm ($$(ls -la dist/trailers.wasm | awk '{print $$5}') bytes)"

space:
	cd plugins/space && cargo build --release --target wasm32-wasip1
	cp plugins/space/target/wasm32-wasip1/release/space_rust.wasm dist/space.wasm
	@echo "Built dist/space.wasm ($$(ls -la dist/space.wasm | awk '{print $$5}') bytes)"

clean:
	rm -rf dist/*.wasm
	cd plugins/space && cargo clean 2>/dev/null || true
	cd plugins/radiogarden && cargo clean 2>/dev/null || true

install:
	@if [ -z "$(MEDIAHUB_PLUGINS_DIR)" ]; then echo "Set MEDIAHUB_PLUGINS_DIR"; exit 1; fi
	cp dist/*.wasm $(MEDIAHUB_PLUGINS_DIR)/
	@echo "Installed $$(ls dist/*.wasm | wc -l | tr -d ' ') plugins to $(MEDIAHUB_PLUGINS_DIR)"
