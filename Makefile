.PHONY: pcfg_tool
pcfg_tool:
	cargo build --release
	cp -p target/release/pcfg_tool .
