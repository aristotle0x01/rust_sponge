test:
	cargo test --all-features
	cd ./build && make check_lab4

clean:
	cargo clean

.PHONY: test clean
