#!/bin/sh

HERE=`dirname $0`
cd "${HERE}"

if [ ! -d blst ]; then
    trap '[ -h blst ] && rm -f blst' 0 2
    ln -s ../.. blst
fi

# --allow-dirty because the temporary blst symbolic link is not committed
cargo +stable publish --allow-dirty "$@"
