#!/bin/sh

run() {
	echo "  Running $*"
	"$@" || exit 1
}

targets_map="arm64 aarch64-unknown-linux-musl
arm arm-unknown-linux-musleabi
amd64 x86_64-unknown-linux-musl"

target="`echo "$targets_map" | awk -v arch="$TARGETARCH" '$1 == arch {print $2}'`"
echo "INFO: BUILDARCH=$BUILDARCH, TARGETARCH=$TARGETARCH, target=$target"
if test -z "$target"; then
	echo "No rustup target known for TARGETARCH=$TARGETARCH" >&2
	exit 1
fi

if test "$BUILDARCH" != "$TARGETARCH"; then
	echo export "CARGO_TARGET_`echo "$target" | tr 'a-z-' 'A-Z_'`_LINKER"="rust-lld"
	export "CARGO_TARGET_`echo "$target" | tr 'a-z-' 'A-Z_'`_LINKER"="rust-lld"


	echo "Adding target $target"
	run rustup target add "$target"
fi

echo "Building for target $target"
run cargo install --target "$target" --path dlc --root dlc_out
run file dlc_out/bin/dlc

