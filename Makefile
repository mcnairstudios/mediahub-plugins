.PHONY: all clean test demo space radiogarden trailers \
	iptvorg somafm radiobrowser librivox aerials nasa peertube trending \
	archive oldtimeradio publicdomain ccmixter outdoorcams naturecams \
	podcasts plutotv skylinecams slowtv operavision trafficcams ted \
	sciencetube cartoons worldnews

RUST_PLUGINS = space radiogarden iptvorg somafm radiobrowser librivox aerials nasa \
	peertube trending archive oldtimeradio publicdomain ccmixter outdoorcams \
	naturecams podcasts plutotv skylinecams slowtv operavision trafficcams ted \
	sciencetube cartoons worldnews

# openmhz disabled — API behind Cloudflare Managed Challenge, unsolvable from server-side HTTP

GO_PLUGINS = demo trailers

all: $(GO_PLUGINS) $(RUST_PLUGINS)
	@echo "All $$(ls dist/*.wasm | wc -l | tr -d ' ') plugins built"

# --- Go plugins (TinyGo) ---

demo:
	cd plugins/demo && tinygo build -o ../../dist/demo.wasm -target=wasi -no-debug ./
	@echo "Built dist/demo.wasm ($$(ls -la dist/demo.wasm | awk '{print $$5}') bytes)"

trailers:
	cd plugins/trailers && tinygo build -o ../../dist/trailers.wasm -target=wasi -no-debug ./
	@echo "Built dist/trailers.wasm ($$(ls -la dist/trailers.wasm | awk '{print $$5}') bytes)"

# --- Rust plugins ---

space:
	cd plugins/space && cargo build --release --target wasm32-wasip1
	cp plugins/space/target/wasm32-wasip1/release/space_rust.wasm dist/space.wasm
	@echo "Built dist/space.wasm ($$(ls -la dist/space.wasm | awk '{print $$5}') bytes)"

radiogarden:
	cd plugins/radiogarden && cargo build --release --target wasm32-wasip1
	cp plugins/radiogarden/target/wasm32-wasip1/release/radiogarden_rust.wasm dist/radiogarden.wasm
	@echo "Built dist/radiogarden.wasm ($$(ls -la dist/radiogarden.wasm | awk '{print $$5}') bytes)"

iptvorg:
	cd plugins/iptvorg && cargo build --release --target wasm32-wasip1
	cp plugins/iptvorg/target/wasm32-wasip1/release/iptvorg_rust.wasm dist/iptvorg.wasm
	@echo "Built dist/iptvorg.wasm ($$(ls -la dist/iptvorg.wasm | awk '{print $$5}') bytes)"

somafm:
	cd plugins/somafm && cargo build --release --target wasm32-wasip1
	cp plugins/somafm/target/wasm32-wasip1/release/somafm_rust.wasm dist/somafm.wasm
	@echo "Built dist/somafm.wasm ($$(ls -la dist/somafm.wasm | awk '{print $$5}') bytes)"

radiobrowser:
	cd plugins/radiobrowser && cargo build --release --target wasm32-wasip1
	cp plugins/radiobrowser/target/wasm32-wasip1/release/radiobrowser_rust.wasm dist/radiobrowser.wasm
	@echo "Built dist/radiobrowser.wasm ($$(ls -la dist/radiobrowser.wasm | awk '{print $$5}') bytes)"

librivox:
	cd plugins/librivox && cargo build --release --target wasm32-wasip1
	cp plugins/librivox/target/wasm32-wasip1/release/librivox_rust.wasm dist/librivox.wasm
	@echo "Built dist/librivox.wasm ($$(ls -la dist/librivox.wasm | awk '{print $$5}') bytes)"

aerials:
	cd plugins/aerials && cargo build --release --target wasm32-wasip1
	cp plugins/aerials/target/wasm32-wasip1/release/aerials_rust.wasm dist/aerials.wasm
	@echo "Built dist/aerials.wasm ($$(ls -la dist/aerials.wasm | awk '{print $$5}') bytes)"

nasa:
	cd plugins/nasa && cargo build --release --target wasm32-wasip1
	cp plugins/nasa/target/wasm32-wasip1/release/nasa_rust.wasm dist/nasa.wasm
	@echo "Built dist/nasa.wasm ($$(ls -la dist/nasa.wasm | awk '{print $$5}') bytes)"

peertube:
	cd plugins/peertube && cargo build --release --target wasm32-wasip1
	cp plugins/peertube/target/wasm32-wasip1/release/peertube_rust.wasm dist/peertube.wasm
	@echo "Built dist/peertube.wasm ($$(ls -la dist/peertube.wasm | awk '{print $$5}') bytes)"

trending:
	cd plugins/trending && cargo build --release --target wasm32-wasip1
	cp plugins/trending/target/wasm32-wasip1/release/trending_rust.wasm dist/trending.wasm
	@echo "Built dist/trending.wasm ($$(ls -la dist/trending.wasm | awk '{print $$5}') bytes)"

archive:
	cd plugins/archive && cargo build --release --target wasm32-wasip1
	cp plugins/archive/target/wasm32-wasip1/release/archive_rust.wasm dist/archive.wasm
	@echo "Built dist/archive.wasm ($$(ls -la dist/archive.wasm | awk '{print $$5}') bytes)"

oldtimeradio:
	cd plugins/oldtimeradio && cargo build --release --target wasm32-wasip1
	cp plugins/oldtimeradio/target/wasm32-wasip1/release/oldtimeradio_rust.wasm dist/oldtimeradio.wasm
	@echo "Built dist/oldtimeradio.wasm ($$(ls -la dist/oldtimeradio.wasm | awk '{print $$5}') bytes)"

publicdomain:
	cd plugins/publicdomain && cargo build --release --target wasm32-wasip1
	cp plugins/publicdomain/target/wasm32-wasip1/release/publicdomain_rust.wasm dist/publicdomain.wasm
	@echo "Built dist/publicdomain.wasm ($$(ls -la dist/publicdomain.wasm | awk '{print $$5}') bytes)"

ccmixter:
	cd plugins/ccmixter && cargo build --release --target wasm32-wasip1
	cp plugins/ccmixter/target/wasm32-wasip1/release/ccmixter_rust.wasm dist/ccmixter.wasm
	@echo "Built dist/ccmixter.wasm ($$(ls -la dist/ccmixter.wasm | awk '{print $$5}') bytes)"

outdoorcams:
	cd plugins/outdoorcams && cargo build --release --target wasm32-wasip1
	cp plugins/outdoorcams/target/wasm32-wasip1/release/outdoorcams_rust.wasm dist/outdoorcams.wasm
	@echo "Built dist/outdoorcams.wasm ($$(ls -la dist/outdoorcams.wasm | awk '{print $$5}') bytes)"

naturecams:
	cd plugins/naturecams && cargo build --release --target wasm32-wasip1
	cp plugins/naturecams/target/wasm32-wasip1/release/naturecams_rust.wasm dist/naturecams.wasm
	@echo "Built dist/naturecams.wasm ($$(ls -la dist/naturecams.wasm | awk '{print $$5}') bytes)"

podcasts:
	cd plugins/podcasts && cargo build --release --target wasm32-wasip1
	cp plugins/podcasts/target/wasm32-wasip1/release/podcasts_rust.wasm dist/podcasts.wasm
	@echo "Built dist/podcasts.wasm ($$(ls -la dist/podcasts.wasm | awk '{print $$5}') bytes)"

plutotv:
	cd plugins/plutotv && cargo build --release --target wasm32-wasip1
	cp plugins/plutotv/target/wasm32-wasip1/release/plutotv_rust.wasm dist/plutotv.wasm
	@echo "Built dist/plutotv.wasm ($$(ls -la dist/plutotv.wasm | awk '{print $$5}') bytes)"

skylinecams:
	cd plugins/skylinecams && cargo build --release --target wasm32-wasip1
	cp plugins/skylinecams/target/wasm32-wasip1/release/skylinecams_rust.wasm dist/skylinecams.wasm
	@echo "Built dist/skylinecams.wasm ($$(ls -la dist/skylinecams.wasm | awk '{print $$5}') bytes)"

slowtv:
	cd plugins/slowtv && cargo build --release --target wasm32-wasip1
	cp plugins/slowtv/target/wasm32-wasip1/release/slowtv_rust.wasm dist/slowtv.wasm
	@echo "Built dist/slowtv.wasm ($$(ls -la dist/slowtv.wasm | awk '{print $$5}') bytes)"

operavision:
	cd plugins/operavision && cargo build --release --target wasm32-wasip1
	cp plugins/operavision/target/wasm32-wasip1/release/operavision_rust.wasm dist/operavision.wasm
	@echo "Built dist/operavision.wasm ($$(ls -la dist/operavision.wasm | awk '{print $$5}') bytes)"

trafficcams:
	cd plugins/trafficcams && cargo build --release --target wasm32-wasip1
	cp plugins/trafficcams/target/wasm32-wasip1/release/trafficcams_rust.wasm dist/trafficcams.wasm
	@echo "Built dist/trafficcams.wasm ($$(ls -la dist/trafficcams.wasm | awk '{print $$5}') bytes)"

ted:
	cd plugins/ted && cargo build --release --target wasm32-wasip1
	cp plugins/ted/target/wasm32-wasip1/release/ted_rust.wasm dist/ted.wasm
	@echo "Built dist/ted.wasm ($$(ls -la dist/ted.wasm | awk '{print $$5}') bytes)"

openmhz:
	cd plugins/openmhz && cargo build --release --target wasm32-wasip1
	cp plugins/openmhz/target/wasm32-wasip1/release/openmhz_rust.wasm dist/openmhz.wasm
	@echo "Built dist/openmhz.wasm ($$(ls -la dist/openmhz.wasm | awk '{print $$5}') bytes)"

sciencetube:
	cd plugins/sciencetube && cargo build --release --target wasm32-wasip1
	cp plugins/sciencetube/target/wasm32-wasip1/release/sciencetube_rust.wasm dist/sciencetube.wasm
	@echo "Built dist/sciencetube.wasm ($$(ls -la dist/sciencetube.wasm | awk '{print $$5}') bytes)"

cartoons:
	cd plugins/cartoons && cargo build --release --target wasm32-wasip1
	cp plugins/cartoons/target/wasm32-wasip1/release/cartoons_rust.wasm dist/cartoons.wasm
	@echo "Built dist/cartoons.wasm ($$(ls -la dist/cartoons.wasm | awk '{print $$5}') bytes)"

# --- Test all Rust plugins ---

test:
	@failed=0; \
	for plugin in $(RUST_PLUGINS); do \
		echo "Testing $$plugin..."; \
		(cd plugins/$$plugin && cargo test 2>&1) || failed=$$((failed + 1)); \
	done; \
	if [ $$failed -gt 0 ]; then echo "$$failed plugin(s) failed tests"; exit 1; fi
	@echo "All Rust plugin tests passed"

# --- Clean ---

clean:
	rm -rf dist/*.wasm
	@for plugin in $(RUST_PLUGINS); do \
		(cd plugins/$$plugin && cargo clean 2>/dev/null) || true; \
	done

# --- Install ---

install:
	@if [ -z "$(MEDIAHUB_PLUGINS_DIR)" ]; then echo "Set MEDIAHUB_PLUGINS_DIR"; exit 1; fi
	cp dist/*.wasm $(MEDIAHUB_PLUGINS_DIR)/
	@echo "Installed $$(ls dist/*.wasm | wc -l | tr -d ' ') plugins to $(MEDIAHUB_PLUGINS_DIR)"

worldnews:
	cd plugins/worldnews && cargo build --release --target wasm32-wasip1
	cp plugins/worldnews/target/wasm32-wasip1/release/worldnews_rust.wasm dist/worldnews.wasm
	@echo "Built dist/worldnews.wasm ($$(ls -la dist/worldnews.wasm | awk '{print $$5}') bytes)"
