#!/bin/bash

cd `dirname $0`
cargo +nightly build -Zbuild-std=core --target x86_64-unknown-uefi --release