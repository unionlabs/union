# This file makes a development and test environment that includes the latest versions of relevant databases.

FROM archlinux

ENV GOPATH /go
ENV PATH $PATH:/go/bin

RUN pacman -Syyu --noconfirm go base-devel rocksdb leveldb git

RUN mkdir /go && \
      chmod -R 777 /go
