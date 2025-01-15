# !/bin/bash

cargo build --release
cp target/release/libpath_reduction.so ../pathAFLplusplus
cp target/release/libpath_reduction.so /home/zekun/modified_magma/fuzzers/pathfuzzerfxhash/fetched_repo/libpath_reduction.so