all:

doc:
	cargo doc --open

run:
	cargo build
	sudo target/debug/zebra
