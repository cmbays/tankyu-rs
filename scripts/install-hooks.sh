#!/bin/sh
# Run once after cloning to wire up the git hooks in .githooks/.
# Usage: sh scripts/install-hooks.sh

git config core.hooksPath .githooks
echo "Git hooks installed."
echo "  pre-commit : fmt check + clippy"
echo "  pre-push   : cargo test --all + unreviewed snapshot check"
