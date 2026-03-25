#!/bin/bash
set -e
set -x
curdir=$(pwd)

# zktls
cd ${curdir}/brevis
RUST_LOG=info cargo pico build

cd ${curdir}/succinct
RUST_LOG=info cargo prove build
mkdir -p bin
cp ./target/elf-compilation/riscv32im-succinct-zkvm-elf/release/program ./bin/program -f
