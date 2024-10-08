#!/bin/bash

version="$1"
if test -z "$version"; then
	echo "Usage: $0 <version>" >&2
	exit 1
fi

#Check for uncommited changes
if ! git diff --quiet; then
	echo "There are uncommited changes in repository." >&2
	git status
	exit 1
fi

#Check version in each cargo.toml
for file in protocol/Cargo.toml rust/Cargo.toml; do
	file_version="`sed -nr -e 's/.*version[ =]*"([^"]*)".*/\1/p' "$file" | head -n 1`"

	if test "$file_version" != "$version"; then
		echo "$file: Version mismatch: change $file_version -> $version" >&2
		exit 1
	fi
done

echo git tag -m "Release $version" "r${version}"
git tag -m "Release $version" "r${version}"


