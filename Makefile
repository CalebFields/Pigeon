CRATE_DIR:=Pigeon

.PHONY: fmt lint test check

fmt:
	cd $(CRATE_DIR) && cargo fmt --all

lint:
	cd $(CRATE_DIR) && cargo clippy --all-targets -- -D warnings

test:
	cd $(CRATE_DIR) && cargo test --verbose

check:
	cd $(CRATE_DIR) && cargo fmt --all -- --check
	cd $(CRATE_DIR) && cargo clippy --all-targets -- -D warnings
	cd $(CRATE_DIR) && cargo test --verbose


