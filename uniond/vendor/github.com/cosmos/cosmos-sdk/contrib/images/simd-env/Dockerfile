FROM golang:1.19-alpine AS build

RUN apk add build-base git linux-headers

WORKDIR /work
COPY go.mod go.sum /work/
COPY errors/go.mod errors/go.sum /work/errors/
COPY math/go.mod math/go.sum /work/math/
COPY api/go.mod api/go.sum /work/api/
COPY core/go.mod core/go.sum /work/core/
COPY depinject/go.mod depinject/go.sum /work/depinject/
COPY tools/rosetta/go.mod tools/rosetta/go.sum /work/tools/rosetta/
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
