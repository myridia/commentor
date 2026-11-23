#!/bin/sh
# id: 4
# group: MAINTENANCE
# desc: Check - cargo check
. "$(dirname "$0")/lib.sh"

enter
cargo check
