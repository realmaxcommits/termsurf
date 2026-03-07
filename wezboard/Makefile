.PHONY: all fmt build check test docs servedocs

all: build

test:
	cargo nextest run
	cargo nextest run -p wezboard-escape-parser # no_std by default

check:
	cargo check
	cargo check -p wezboard-escape-parser
	cargo check -p wezboard-cell
	cargo check -p wezboard-surface
	cargo check -p wezboard-ssh

build:
	cargo build $(BUILD_OPTS) -p wezboard
	cargo build $(BUILD_OPTS) -p wezboard-gui
	cargo build $(BUILD_OPTS) -p wezboard-mux-server
	cargo build $(BUILD_OPTS) -p strip-ansi-escapes

fmt:
	cargo +nightly fmt

docs:
	ci/build-docs.sh

servedocs:
	ci/build-docs.sh serve
