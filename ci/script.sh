set -euxo pipefail

main() {
    cargo check --target $TARGET

    cargo check --target $TARGET --features unstable
    cargo test --target $TARGET --features unstable
}

main
