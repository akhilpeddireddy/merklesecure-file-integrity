fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

demo:
	cargo test --test client_server_integration -- --nocapture