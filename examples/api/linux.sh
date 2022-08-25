#!/bin/bash
# build
cargo b && mkdir lua -p && mv target/debug/libapi.so lua/api.so -fn
# nvim
set_rtp=":set rtp+=$PWD"
cmd="
:lua require'api'
"
RUST_BACKTRACE=1 nvim -u NONE --headless +"$set_rtp" +"$cmd" +quit
