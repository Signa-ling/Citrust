#!/bin/bash

cd `dirname $0`
cargo +nightly build -Zbuild-std=core --target x86-64-CitrustOS.json --release