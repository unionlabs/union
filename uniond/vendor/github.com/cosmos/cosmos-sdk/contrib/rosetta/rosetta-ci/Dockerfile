FROM golang:1.19-alpine as build

RUN apk add git tar gcc libc-dev --no-cache

ARG ROSETTA_VERSION="v0.10.0"

# build rosetta CLI
WORKDIR /rosetta
RUN git clone https://github.com/coinbase/rosetta-cli . && \
    git checkout tags/$ROSETTA_VERSION && \
    go build -o rosetta-cli ./main.go

# prepare node data
WORKDIR /node
COPY ./contrib/rosetta/rosetta-ci/data.tar.gz data.tar.gz
RUN tar -zxvf data.tar.gz -C .

# build simd
WORKDIR /simd
COPY . ./
RUN cd simapp && go build -o simd ./simd/

FROM alpine
RUN apk add gcc git libc-dev python3 --no-cache

ENV PATH=$PATH:/bin
# Set GENESIS_HASH env needed for api correctness
ENV GENESIS_HASH "01331100220a94b9acc0ceef697a6db44ba70fc61e9e6c24802974a0b6cba29b"

COPY --from=build /rosetta/rosetta-cli /bin/rosetta-cli
COPY --from=build /simd/simapp/simd/simd /bin/simd

WORKDIR /rosetta
COPY ./contrib/rosetta/configuration ./
RUN chmod +x run_tests.sh && \
    chmod +x send_funds.sh && \
    chmod +x faucet.py

COPY --from=build /node/root /root/
WORKDIR /root/.simapp

RUN chmod -R 0777 ./
