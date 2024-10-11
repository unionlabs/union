#! /bin/bash

export GO111MODULE=on

set -u

#####################
# counter over socket
#####################
TESTNAME=$1

# Send some txs

function getCode() {
	set +u
	R=$1
	set -u
	if [[ "$R" == "" ]]; then
		echo -1
	fi

	if [[ $(echo $R | jq 'has("code")') == "true" ]]; then
		# this won't actually work if there's an error ...
		echo "$R" | jq ".code"
	else
		# protobuf auto adds `omitempty` to everything so code OK and empty data/log
		# will not even show when marshalled into json
		# apparently we can use github.com/golang/protobuf/jsonpb to do the marshalling ...
		echo 0
	fi
}

function sendTx() {
	TX=$1
	set +u
	SHOULD_ERR=$2
	if [ "$SHOULD_ERR" == "" ]; then
		SHOULD_ERR=false
	fi
	set -u
    RESPONSE=$(curl -s localhost:26657/broadcast_tx_commit?tx=0x"$TX")
    IS_ERR=$(echo "$RESPONSE" | jq 'has("error")')
    ERROR=$(echo "$RESPONSE" | jq '.error')
    ERROR=$(echo "$ERROR" | tr -d '"') # remove surrounding quotes

    RESPONSE=$(echo "$RESPONSE" | jq '.result')

	echo "RESPONSE"
	echo "$RESPONSE"

	echo "$RESPONSE" | jq . &> /dev/null
	IS_JSON=$?
	if [[ "$IS_JSON" != "0" ]]; then
		IS_ERR=true
		ERROR="$RESPONSE"
	fi
	APPEND_TX_RESPONSE=$(echo "$RESPONSE" | jq '.deliver_tx')
	APPEND_TX_CODE=$(getCode "$APPEND_TX_RESPONSE")
	CHECK_TX_RESPONSE=$(echo "$RESPONSE" | jq '.check_tx')
	CHECK_TX_CODE=$(getCode "$CHECK_TX_RESPONSE")

	echo "-------"
	echo "TX $TX"
	echo "RESPONSE $RESPONSE"
	echo "ERROR $ERROR"
	echo "IS_ERR $IS_ERR"
	echo "----"

	if $SHOULD_ERR; then
		if [[ "$IS_ERR" != "true" ]]; then
			echo "Expected error sending tx ($TX)"
			exit 1
		fi
	else
		if [[ "$IS_ERR" == "true" ]]; then
			echo "Unexpected error sending tx ($TX)"
			exit 1
		fi

	fi
}

echo "... sending tx. expect no error"

# 0 should pass once and get in block, with no error
TX=00
sendTx $TX
if [[ $APPEND_TX_CODE != 0 ]]; then
	echo "Got non-zero exit code for $TX. $RESPONSE"
	exit 1
fi


echo "... sending tx. expect error"

# second time should get rejected by the mempool (return error and non-zero code)
sendTx $TX true


echo "... sending tx. expect no error"

# now, TX=01 should pass, with no error
TX=01
sendTx $TX
if [[ $APPEND_TX_CODE != 0 ]]; then
	echo "Got non-zero exit code for $TX. $RESPONSE"
	exit 1
fi

echo "... sending tx. expect no error, but invalid"

# now, TX=03 should get in a block (passes CheckTx, no error), but is invalid
TX=03
sendTx $TX
if [[ "$CHECK_TX_CODE" != 0 ]]; then
	echo "Got non-zero exit code for checktx on $TX. $RESPONSE"
	exit 1
fi
if [[ $APPEND_TX_CODE == 0 ]]; then
	echo "Got zero exit code for $TX. Should have been bad nonce. $RESPONSE"
	exit 1
fi

echo "Passed Test: $TESTNAME"
