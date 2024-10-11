FROM martenseemann/quic-network-simulator-endpoint:latest AS builder

ARG TARGETPLATFORM
RUN echo "TARGETPLATFORM: ${TARGETPLATFORM}"

RUN apt-get update && apt-get install -y wget tar git

ENV GOVERSION=1.21.1

RUN platform=$(echo ${TARGETPLATFORM} | tr '/' '-') && \
  filename="go${GOVERSION}.${platform}.tar.gz" && \
  wget --no-verbose https://dl.google.com/go/${filename} && \
  tar xfz ${filename} && \
  rm ${filename}

ENV PATH="/go/bin:${PATH}"

RUN git clone https://go.googlesource.com/net

WORKDIR /net
RUN go build -o /interop ./internal/quic/cmd/interop

FROM martenseemann/quic-network-simulator-endpoint:latest

WORKDIR /go-x-net

COPY --from=builder /interop ./

# copy run script and run it
COPY run_endpoint.sh .
RUN chmod +x run_endpoint.sh
ENTRYPOINT [ "./run_endpoint.sh" ]
