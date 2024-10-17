#!/bin/sh

#TODO: Add support for other targets

targets_map="arm64 aarch64-unknown-linux-musl
arm arm-unknown-linux-musleabi
amd64 x86_64-unknown-linux-musl"

target="`echo "$targets_map" | awk -v arch="$TARGETARCH" '$1 == arch {print $2}'`"

if test -z "$target"; then
	echo "No rustup target known for TARGETARCH=$TARGETARCH" >&2
	exit 1
fi

run() {
	echo "  Running $*"
	"$@" || exit 1
}

echo "Adding target $target"
run rustup target add "$target"

echo "Building for target $target"
run cargo install --target "$target" --path dlc --root dlc_out
run file dlc_out/dlc

