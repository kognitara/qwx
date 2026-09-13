all: before
	@cargo build --release
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
