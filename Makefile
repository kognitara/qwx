all: before
	@cargo build --release --no-default-features -F tree-sitter-nix -F tree-sitter-asm -F tree-sitter-c -F tree-sitter-lua -F tree-sitter-cpp -F tree-sitter-d -F tree-sitter-rust -F tree-sitter-bash -F tree-sitter-python -F tree-sitter-cmake -F tree-sitter-make -F tree-sitter-kconfig -F tree-sitter-ini -F tree-sitter-json -F tree-sitter-fish
install: completion
	@install -m 755 target/release/qwx /usr/local/bin/qwx
	@install -m 755 target/release/we /usr/local/bin/we
	@install -m 644 qwx.fish /etc/fish/completions/qwx.fish
	@install -m 644 we.fish /etc/fish/completions/we.fish
uninstall:
	@rm /usr/local/bin/qwx
	@rm /etc/fish/completions/qwx.fish
	@rm /etc/fish/completions/we.fish
completion:
	@target/release/qwx gen fish > qwx.fish
	@target/release/we gen fish > we.fish
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
