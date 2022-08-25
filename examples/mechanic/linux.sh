#!/bin/bash

# build
if [ $RELEASE ]; then
    cargo b -r -q && mkdir lua -p && cp ../target/release/libmechanic.so lua/mechanic.so -f
    echo "release built"
else
    cargo b -q && mkdir lua -p && cp ../target/debug/libmechanic.so lua/mechanic.so -f;
    echo "debug built"
fi

# nvim
lua='
local mechanic = require("mechanic")

local fixed = mechanic.fix({
  manufacturer = "Tesla",
  miles = 69420,
  works = false,
  problem = "kills_pedestrians",
})

assert(fixed.works)
assert(fixed.problem == nil)
print(vim.inspect(fixed))
'
set_rtp=":set rtp+=$PWD"
RUST_BACKTRACE=1 nvim -u NONE --headless +"$set_rtp" +":lua $lua" +quit
