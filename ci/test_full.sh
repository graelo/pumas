#!/bin/bash

set -e

CRATE=pumas
MSRV=1.95

get_rust_version() {
  local array=($(rustc --version));
  echo "${array[1]}";
  return 0;
}
RUST_VERSION=$(get_rust_version)

check_version() {
  IFS=. read -ra rust <<< "$RUST_VERSION"
  IFS=. read -ra want <<< "$1"
  [[ "${rust[0]}" -gt "${want[0]}" ||
   ( "${rust[0]}" -eq "${want[0]}" &&
     "${rust[1]}" -ge "${want[1]}" )
  ]]
}

echo "Testing $CRATE on rustc $RUST_VERSION"
if ! check_version $MSRV ; then
  echo "The minimum for $CRATE is rustc $MSRV"
  exit 1
fi

# List crate-specific features here (none for pumas today).
FEATURES=()
echo "Testing supported features: ${FEATURES[*]}"

NEXTEST_PROFILE=""
if [ -n "$CI" ]; then
  NEXTEST_PROFILE="--profile ci"
fi

set -x

# test the default
cargo build --locked
cargo nextest run --locked $NEXTEST_PROFILE

# test --no-default-features
cargo build --locked --no-default-features
cargo nextest run --locked $NEXTEST_PROFILE --no-default-features

# test each feature in isolation
for feature in "${FEATURES[@]}"; do
  cargo build --locked --no-default-features --features="$feature"
  cargo nextest run --locked $NEXTEST_PROFILE --no-default-features --features="$feature"
done

# test all features combined
cargo build --locked --features="${FEATURES[*]}"
cargo nextest run --locked $NEXTEST_PROFILE --features="${FEATURES[*]}"

# CLI smoke test (release binary, target-aware path)
cargo build --locked --release

BIN="target/${CARGO_BUILD_TARGET:+${CARGO_BUILD_TARGET}/}release/pumas"
case "${OSTYPE:-}" in
  msys*|cygwin*) BIN="${BIN}.exe" ;;
esac
"${BIN}" --help

# doc tests (not supported by nextest)
cargo test --locked --doc
