#!/bin/bash

cargo check && cargo fmt && cargo test && cargo run 2>/tmp/zeditor-err.log
