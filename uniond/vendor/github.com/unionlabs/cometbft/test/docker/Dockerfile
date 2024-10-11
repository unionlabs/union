FROM golang:1.22.5

# Grab deps (jq, hexdump, xxd, killall)
RUN apt-get update && \
  apt-get install -y --no-install-recommends \
  jq bsdmainutils vim-common psmisc netcat-traditional curl

# Setup CometBFT repo
ENV REPO $GOPATH/src/github.com/cometbft/cometbft
ENV GOBIN $GOPATH/bin
WORKDIR $REPO

# Copy in the code
# TODO: rewrite to only copy Makefile & other files?
COPY . $REPO


# install ABCI CLI
RUN make install_abci

# install CometBFT
RUN make install

RUN cometbft testnet \
  --config $REPO/test/docker/config-template.toml \
  --node-dir-prefix="mach" \
  --v=4 \
  --populate-persistent-peers=false \
  --o=$REPO/test/p2p/data

# Now copy in the code
# NOTE: this will overwrite whatever is in vendor/
COPY . $REPO

# expose the volume for debugging
VOLUME $REPO

EXPOSE 26656
EXPOSE 26657
