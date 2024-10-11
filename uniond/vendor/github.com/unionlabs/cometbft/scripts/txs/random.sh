#! /bin/bash
set -u

function toHex() {
	echo -n $1 | hexdump -ve '1/1 "%.2X"'
}

N=$1
PORT=$2

for i in `seq 1 $N`; do
	# store key value pair
	KEY=$(head -c 10 /dev/urandom)
	VALUE="$i"
	echo $(toHex $KEY=$VALUE)
	curl 127.0.0.1:$PORT/broadcast_tx_sync?tx=0x$(toHex $KEY=$VALUE)
done


