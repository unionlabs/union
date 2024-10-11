FROM golang:1.23-alpine AS build

RUN apk add build-base git linux-headers

WORKDIR /work
COPY go.mod go.sum /work/
COPY math/go.mod math/go.sum /work/math/
COPY api/go.mod api/go.sum /work/api/
COPY log/go.mod log/go.sum /work/log/
COPY core/go.mod core/go.sum /work/core/
COPY collections/go.mod collections/go.sum /work/collections/
COPY depinject/go.mod depinject/go.sum /work/depinject/
COPY x/tx/go.mod x/tx/go.sum /work/x/tx/
COPY x/protocolpool/go.mod x/protocolpool/go.sum /work/x/protocolpool/
COPY x/gov/go.mod x/gov/go.sum /work/x/gov/
COPY x/distribution/go.mod x/distribution/go.sum /work/x/distribution/
COPY x/slashing/go.mod x/slashing/go.sum /work/x/slashing/
COPY x/staking/go.mod x/staking/go.sum /work/x/staking/
COPY x/auth/go.mod x/auth/go.sum /work/x/auth/
COPY x/authz/go.mod x/authz/go.sum /work/x/authz/
COPY x/bank/go.mod x/bank/go.sum /work/x/bank/
COPY x/mint/go.mod x/mint/go.sum /work/x/mint/
COPY x/consensus/go.mod x/consensus/go.sum /work/x/consensus/
COPY x/accounts/go.mod x/accounts/go.sum /work/x/accounts/
COPY runtime/v2/go.mod runtime/v2/go.sum /work/runtime/v2/
COPY server/v2/cometbft/go.mod server/v2/cometbft/go.sum /work/server/v2/cometbft/
COPY core/testing/go.mod core/testing/go.sum /work/core/testing/

RUN go mod download

COPY ./ /work
RUN LEDGER_ENABLED=false make clean build


FROM alpine AS run
RUN apk add bash curl jq
EXPOSE 26656 26657
ENTRYPOINT ["/usr/bin/wrapper.sh"]
CMD ["start", "--log_format", "plain"]
STOPSIGNAL SIGTERM
VOLUME /simd
WORKDIR /simd

COPY contrib/images/simd-env/wrapper.sh /usr/bin/wrapper.sh
COPY --from=build /work/build/simd /simd/
