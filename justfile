default:
    @just --list

dev-server:
    RUST_LOG=info cargo watch -x 'run -p faze-cli -- serve'

dev-ui:
    cd ui && bun dev

dev:
    just dev-server &
    just dev-ui

build-ui:
    cd ui && bun run build

build: build-ui
    cargo build --release -p faze-cli

test:
    cargo test --workspace
    cd ui && bun run test

test-ui:
    cd ui && bun run test:ui

test-ui-coverage:
    cd ui && bun run test:coverage

check:
    cargo clippy --workspace
    cargo fmt --check
    cd ui && bun run lint

clean:
    cargo clean
    rm -rf ui/node_modules ui/dist
