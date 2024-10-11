#!/usr/bin/env bash

DAEMON_HOME="/tmp/simd$(date +%s)"
RANDOM_KEY="randomvalidatorkey"

echo "#############################################"
echo "### Ensure to set the below ENV settings ###"
echo "#############################################"
echo "
DAEMON= # ex: simd
CHAIN_ID= # ex: testnet-1
DENOM= # ex: ustake
GH_URL= # ex: https://github.com/cosmos/cosmos-sdk
BINARY_VERSION= # ex :v0.44.0
GO_VERSION=1.17
PRELAUNCH_GENESIS_URL= # ex: https://raw.githubusercontent.com/cosmos/cosmos-sdk/master/$CHAIN_ID/genesis-prelaunch.json
GENTXS_DIR= # ex: $GOPATH/github.com/cosmos/mainnet/$CHAIN_ID/gentxs"
echo

if [[ -z "${GH_URL}" ]]; then
  echo "GH_URL in not set, required. Ex: https://github.com/cosmos/cosmos-sdk"
  exit 0
fi
if [[ -z "${DAEMON}" ]]; then
  echo "DAEMON is not set, required. Ex: simd, gaiad etc"
  exit 0
fi
if [[ -z "${DENOM}" ]]; then
  echo "DENOM in not set, required. Ex: stake, uatom etc"
  exit 0
fi
if [[ -z "${GO_VERSION}" ]]; then
  echo "GO_VERSION in not set, required. Ex: 1.15.2, 1.16.6 etc."
  exit 0
fi
if [[ -z "${CHAIN_ID}" ]]; then
  echo "CHAIN_ID in not set, required."
  exit 0
fi
if [[ -z "${PRELAUNCH_GENESIS_URL}" ]]; then
  echo "PRELAUNCH_GENESIS_URL (genesis file url) in not set, required."
  exit 0
fi
if [[ -z "${GENTXS_DIR}" ]]; then
  echo "GENTXS_DIR in not set, required."
  exit 0
fi

command_exists () {
    type "$1" &> /dev/null ;
}

if command_exists go ; then
    echo "Golang is already installed"
else
  read -s -p "Installing go using apt. Do you want to proceed (y/n)?: " useApt

  if [ "$useApt" != "y" ]; then
    echo
    echo "Install go manually and execute this script"
    exit 0;
  fi

  sudo apt update
  sudo apt install build-essential -y

  wget https://dl.google.com/go/go$GO_VERSION.linux-amd64.tar.gz
  tar -xvf go$GO_VERSION.linux-amd64.tar.gz
  sudo mv go /usr/local

  echo "" >> ~/.profile
  echo 'export GOPATH=$HOME/go' >> ~/.profile
  echo 'export GOROOT=/usr/local/go' >> ~/.profile
  echo 'export GOBIN=$GOPATH/bin' >> ~/.profile
  echo 'export PATH=$PATH:/usr/local/go/bin:$GOBIN' >> ~/.profile

  . ~/.profile

  go version
fi

if [ "$(ls -A $GENTXS_DIR)" ]; then
    echo "Install $DAEMON"
    git clone $GH_URL $DAEMON
    cd $DAEMON
    git fetch && git checkout $BINARY_VERSION
    make install
    $DAEMON version

    for GENTX_FILE in $GENTXS_DIR/*.json; do
        if [ -f "$GENTX_FILE" ]; then
            set -e

            echo "GentxFile::::"
            echo $GENTX_FILE

            echo "...........Init a testnet.............."
            $DAEMON init --chain-id $CHAIN_ID validator --home $DAEMON_HOME

            $DAEMON keys add $RANDOM_KEY --keyring-backend test --home $DAEMON_HOME

            echo "..........Fetching genesis......."
            curl -s $PRELAUNCH_GENESIS_URL > $DAEMON_HOME/config/genesis.json

            # this genesis time is different from original genesis time, just for validating gentx.
            sed -i '/genesis_time/c\   \"genesis_time\" : \"2021-01-01T00:00:00Z\",' $DAEMON_HOME/config/genesis.json

            GENACC=$(cat $GENTX_FILE | sed -n 's|.*"delegator_address":"\([^"]*\)".*|\1|p')
            denomquery=$(jq -r '.body.messages[0].value.denom' $GENTX_FILE)
            amountquery=$(jq -r '.body.messages[0].value.amount' $GENTX_FILE)

            # only allow $DENOM tokens to be bonded
            if [ $denomquery != $DENOM ]; then
                echo "invalid denomination"
                exit 1
            fi

            $DAEMON add-genesis-account $RANDOM_KEY 1000000000000000$DENOM --home $DAEMON_HOME \
                --keyring-backend test

            $DAEMON gentx $RANDOM_KEY 900000000000000$DENOM --home $DAEMON_HOME \
                --keyring-backend test --chain-id $CHAIN_ID

            cp $GENTX_FILE $DAEMON_HOME/config/gentx/

            echo "..........Collecting gentxs......."
            $DAEMON collect-gentxs --home $DAEMON_HOME
            $DAEMON validate-genesis --home $DAEMON_HOME

            echo "..........Starting node......."
            $DAEMON start --home $DAEMON_HOME &

            sleep 10s

            echo "...checking network status.."
            echo "if this fails, most probably the gentx with address $GENACC is invalid"
            $DAEMON status --node http://localhost:26657

            echo "...Cleaning the stuff..."
            killall $DAEMON >/dev/null 2>&1
            sleep 2s
            rm -rf $DAEMON_HOME
        fi
    done
else
    echo "$GENTXS_DIR is empty, nothing to validate"
fi
