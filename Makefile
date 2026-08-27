all:
	@cargo install --path .
commit:
	@git status
	@git diff --stat
	@sleep 2
	@git diff -p
	@read -p "Commit message: " message
	@git add .
	@git commit -m "$message"
	@git push --all
	@git push --tags