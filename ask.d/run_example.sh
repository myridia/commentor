#!/bin/sh
# id: 3
# group: RUN
# desc: Run example - open example/hello.rs with its comments
. "$(dirname "$0")/lib.sh"

enter
cargo run -- example/hello.rs
