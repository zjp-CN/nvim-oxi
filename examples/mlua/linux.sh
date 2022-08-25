# build
cargo b -r && mkdir lua -p && mv target/release/liblua.so lua/lua.so -fn
# nvim
set_rtp=":set rtp+=$PWD"
cmd=":lua require'lua'.greetings()"
nvim -u NONE --headless +"$set_rtp" +"$cmd" +quit
