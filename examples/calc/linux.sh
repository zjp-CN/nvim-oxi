#!/bin/bash
# build
cargo b -r && mkdir lua -p && mv target/release/libcalc.so lua/calc.so -fn
# nvim
set_rtp=":set rtp+=$PWD"
add="add(-1, 128)"
mul="multiply(-1, 128)"
cpt="compute(calc.multiply, 0, 128)"
cmd="
:lua
  calc=require'calc' add=calc.$add mul=calc.$mul cpt=calc.$cpt
  print('Result:', '\n', '$add:', add, '\n', '$mul:', mul, '\n', '$cpt:', cpt, '\n')
"
nvim -u NONE --headless +"$set_rtp" +"$cmd" +quit
