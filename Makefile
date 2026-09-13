all: before
	@cargo build --release --no-default-features -F tree-sitter-c -F tree-sitter-cpp -F tree-sitter-d -F tree-sitter-rust -F tree-sitter-bash -F tree-sitter-python -F tree-sitter-make -F tree-sitter-kconfig -F tree-sitter-ini -F tree-sitter-json -F tree-sitter-fish
install: completion
	@install -m 755 target/release/qwx /usr/local/bin/qwx
	@install -m 644 qwx.fish /etc/fish/completions/qwx.fish
uninstall:
	@rm /usr/local/bin/qwx
	@rm /etc/fish/completions/qwx.fish
completion:
	@target/release/qwx gen fish > qwx.fish
run:
	@cargo run
commit:
	@git status
	@git diff --stat
	@sleep 2
	@git diff -p
	@git add .
	@git commit
	@git push --all
	@git push --tags
before:
	@cargo fmt
doc:
	@cargo doc --open
