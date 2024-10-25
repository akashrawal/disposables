#!/bin/bash

source ./dev/version.sh || exit 1

#Check for uncommited changes
if ! git diff --quiet; then
	echo "There are uncommited changes in repository." >&2
	git status
	exit 1
fi

echo git tag -m "Release $version" "r${version}"
git tag -m "Release $version" "r${version}"


