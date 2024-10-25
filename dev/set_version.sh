#!/bin/bash

source ./dev/version.sh || exit 1

sed -i -e 's/99.99.99/'"$version"'/g' \
	protocol/Cargo.toml  \
	rust/Cargo.toml  \
	java/build.gradle \
