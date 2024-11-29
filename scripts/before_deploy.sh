#!/usr/bin/env bash
# Build and package for release

set -ex

gcc_prefix=""

tempdir=$(mktemp -d 2>/dev/null || mktemp -d -t tmp)
out_dir=$(pwd)
package_name="$PROJECT_NAME-$TAG-$TARGET"

if [[ $TARGET == "arm-unknown-linux-gnueabihf" ]]; then
	gcc_prefix="arm-linux-gnueabihf-"
elif [[ $TARGET == "aarch64-unknown-linux-gnu" ]]; then
	gcc_prefix="aarch64-linux-gnu-"
else
	gcc_prefix=""
fi

# Create a "staging" directory
mkdir "$tempdir/$package_name"

# Copy the main binary
cp "target/$TARGET/release/$BIN_NAME" "$tempdir/$package_name/"
if [ "$OS_NAME" != windows-latest ]; then
	"${gcc_prefix}"strip "$tempdir/$package_name/$BIN_NAME"
fi

# Copy RADME and LINCENSE
cp README.md "$tempdir/$package_name"
cp LICENSE "$tempdir/$package_name"

# Archive
pushd "$tempdir"
if [ "$OS_NAME" = windows-latest ]; then
	7z a "$out_dir/$package_name.zip" "$package_name"/*
else
	tar czf "$out_dir/$package_name.tar.gz" "$package_name"/*
fi
popd
rm -r "$tempdir"
