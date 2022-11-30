#!/bin/bash

cd adding-machine && cargo build --target wasm32-unknown-unknown

cd ..

cd runrunrun && cargo build

cargo run
