# We need to build in a Linux environment to support C libraries, e.g. RocksDB.
# We use Debian instead of Alpine, so that we can use binary database packages
# instead of spending time compiling them.
FROM cometbft/cometbft-db-testing:v0.14.0

RUN apt-get -qq update -y && apt-get -qq upgrade -y >/dev/null

# For the latency-setter script
RUN apt-get -qq install -y iputils-ping iproute2 python3-netifaces >/dev/null
RUN mkdir -p /scripts
COPY test/e2e/pkg/latency/aws-latencies.csv /scripts/
COPY test/e2e/pkg/latency/latency-setter.py /scripts/

# Set up build directory /src/cometbft
ENV COMETBFT_BUILD_OPTIONS badgerdb,rocksdb,pebbledb,clock_skew,bls12381
WORKDIR /src/cometbft

# Fetch dependencies separately (for layer caching)
COPY go.mod go.sum api/go.mod api/go.sum ./
RUN go mod download

# Build CometBFT and install into /usr/bin/cometbft
COPY . .
RUN make build && cp build/cometbft /usr/bin/cometbft
COPY test/e2e/docker/entrypoint* /usr/bin/
RUN cd test/e2e && make node && cp build/node /usr/bin/app

# Set up runtime directory. We don't use a separate runtime image since we need
# e.g. leveldb and rocksdb which are already installed in the build image.
WORKDIR /cometbft
VOLUME /cometbft
ENV CMTHOME=/cometbft
ENV GORACE "halt_on_error=1"

EXPOSE 26656 26657 26660 6060
ENTRYPOINT ["/usr/bin/entrypoint"]
CMD ["node"]
STOPSIGNAL SIGTERM
