#!/bin/sh
# id: 5
# group: MAINTENANCE
# desc: Clean - cargo clean
. "$(dirname "$0")/lib.sh"

enter
cargo clean
