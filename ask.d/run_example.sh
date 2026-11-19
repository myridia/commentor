#!/bin/sh
# id: 3
# group: RUN
# desc: Run example - open src/main.rs with its comments
. "$(dirname "$0")/lib.sh"

enter
cargo run -- src/main.rs
