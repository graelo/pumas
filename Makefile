# Local task runner for pre-push / pre-PR verification.
#
# Usage:
#   make check       # fmt + lint + test  (run before `git push`)
#   make check-all   # adds audits, commit lint, docs, and CI security checks
#   make fix         # auto-format and apply clippy fixes
#
# The check targets intentionally mirror .github/workflows/ci-essentials.yml so
# a green local run predicts a green CI run. They assume their external tools
# (cargo-nextest, cargo-deny, cargo-pants, convco, poutine, zizmor, rumdl,
# mandoc, cargo-llvm-cov) are already installed locally.

.DEFAULT_GOAL := help

.PHONY: help fmt lint test check audit commits ci-security md man check-all \
	fix release coverage

help:  ## List available targets
	@awk 'BEGIN {FS = ":.*## "} /^[a-zA-Z_-]+:.*## / \
		{printf "  \033[36m%-12s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

fmt:  ## rustfmt --check (no changes)
	cargo fmt --all -- --check

lint:  ## clippy with warnings denied
	cargo clippy --locked --all-targets -- -D warnings

test:  ## full test suite (build + nextest + doc tests)
	./ci/test_full.sh

check: fmt lint test  ## pre-push gate: fmt + lint + test

audit:  ## cargo-deny & cargo-pants: advisories, licenses, bans, sources
	cargo deny --locked check
	cargo pants

commits:  ## verify commit messages follow Conventional Commits
	convco check -c .convco

ci-security:  ## audit GitHub Actions workflows
	poutine --fail-on-violation analyze_local .
	zizmor .github

md:  ## lint Markdown against rumdl.toml
	rumdl check .

man:  ## lint the roff manpage
	mandoc -Tlint man/pumas.1

check-all: check audit commits ci-security md man  ## pre-PR gate: everything

fix:  ## auto-fix: rustfmt + clippy --fix
	cargo fmt --all
	cargo clippy --locked --all-targets --fix --allow-dirty --allow-staged -- -D warnings

release:  ## release build with native CPU optimizations
	RUSTFLAGS="-Ctarget-cpu=native" cargo build --release

coverage:  ## HTML coverage report at target/llvm-cov/html/index.html
	cargo llvm-cov --locked --html
