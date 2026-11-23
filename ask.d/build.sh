#!/bin/sh
# id: 1
# group: SETUP
# desc: Build - cargo build
. "$(dirname "$0")/lib.sh"

enter
cargo build
