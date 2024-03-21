# !/bin/bash

cargo build --release
cp target/release/libpath_reduction.so ../pathAFLplusplus
