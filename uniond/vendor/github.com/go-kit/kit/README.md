# Go kit

![GitHub Workflow Status](https://github.com/go-kit/kit/workflows/CI/badge.svg)
[![GoDev](https://img.shields.io/badge/go.dev-reference-007d9c?logo=go&logoColor=white&style=flat-square)](https://pkg.go.dev/github.com/go-kit/kit?tab=doc)
[![codecov](https://codecov.io/gh/go-kit/kit/branch/master/graph/badge.svg)](https://codecov.io/gh/go-kit/kit)
[![Go Report Card](https://goreportcard.com/badge/go-kit/kit)](https://goreportcard.com/report/go-kit/kit)
[![Sourcegraph](https://sourcegraph.com/github.com/go-kit/kit/-/badge.svg)](https://sourcegraph.com/github.com/go-kit/kit?badge)

**Go kit** is a **programming toolkit** for building microservices
(or elegant monoliths) in Go. We solve common problems in distributed
systems and application architecture so you can focus on delivering
business value.

- Website: [gokit.io](https://gokit.io)
- Mailing list: [go-kit](https://groups.google.com/forum/#!forum/go-kit)
- Slack: [gophers.slack.com](https://gophers.slack.com) **#go-kit** ([invite](https://gophersinvite.herokuapp.com/))

## Sponsors

Click [here](https://github.com/sponsors/peterbourgon) or Sponsor, above, for more information on sponsorship.

## Motivation

Go has emerged as the language of the server, but it remains underrepresented
in so-called "modern enterprise" companies like Facebook, Twitter, Netflix, and
SoundCloud. Many of these organizations have turned to JVM-based stacks for
their business logic, owing in large part to libraries and ecosystems that
directly support their microservice architectures.

To reach its next level of success, Go needs more than simple primitives and
idioms. It needs a comprehensive toolkit, for coherent distributed programming
in the large. Go kit is a set of packages and best practices, which provide a
comprehensive, robust, and trustable way of building microservices for
organizations of any size.

For more details, see
 [the website](https://gokit.io),
 [the motivating blog post](http://peter.bourgon.org/go-kit/) and
 [the video of the talk](https://www.youtube.com/watch?v=iFR_7AKkJFU).
See also the
 [Go kit talk at GopherCon 2015](https://www.youtube.com/watch?v=1AjaZi4QuGo).

## Goals

- Operate in a heterogeneous SOA — expect to interact with mostly non-Go-kit services
- RPC as the primary messaging pattern
- Pluggable serialization and transport — not just JSON over HTTP
- Operate within existing infrastructures — no mandates for specific tools or technologies

## Non-goals

- Supporting messaging patterns other than RPC (for now) — e.g. MPI, pub/sub, CQRS, etc.
- Re-implementing functionality that can be provided by adapting existing software
- Having opinions on operational concerns: deployment, configuration, process supervision, orchestration, etc.

## Contributing

Please see [CONTRIBUTING.md](/CONTRIBUTING.md).
Thank you, [contributors](https://github.com/go-kit/kit/graphs/contributors)!

## Dependency management

Go kit is [modules](https://github.com/golang/go/wiki/Modules) aware, and we
encourage users to use the standard modules tooling. But Go kit is at major
version 0, so it should be compatible with non-modules environments.

## Code generators

There are several third-party tools that can generate Go kit code based on
different starting assumptions.

- [RecoLabs/microgen](https://github.com/RecoLabs/microgen)
- [GrantZheng/kit](https://github.com/GrantZheng/kit)
- [kujtimiihoxha/kit](https://github.com/kujtimiihoxha/kit) (unmaintained)
- [nytimes/marvin](https://github.com/nytimes/marvin)
- [sagikazarmark/mga](https://github.com/sagikazarmark/mga)
- [sagikazarmark/protoc-gen-go-kit](https://github.com/sagikazarmark/protoc-gen-go-kit)
- [metaverse/truss](https://github.com/metaverse/truss)

## Related projects

Projects with a ★ have had particular influence on Go kit's design (or vice-versa).

### Service frameworks

- [gizmo](https://github.com/nytimes/gizmo), a microservice toolkit from The New York Times ★
- [go-micro](https://github.com/micro/go-micro), a distributed systems development framework ★
- [gotalk](https://github.com/rsms/gotalk), async peer communication protocol &amp; library
- [Kite](https://github.com/koding/kite), a micro-service framework
- [gocircuit](https://github.com/gocircuit/circuit), dynamic cloud orchestration

### Individual components

- [afex/hystrix-go](https://github.com/afex/hystrix-go), client-side latency and fault tolerance library
- [armon/go-metrics](https://github.com/armon/go-metrics), library for exporting performance and runtime metrics to external metrics systems
- [codahale/lunk](https://github.com/codahale/lunk), structured logging in the style of Google's Dapper or Twitter's Zipkin
- [eapache/go-resiliency](https://github.com/eapache/go-resiliency), resiliency patterns
- [sasbury/logging](https://github.com/sasbury/logging), a tagged style of logging
- [grpc/grpc-go](https://github.com/grpc/grpc-go), HTTP/2 based RPC
- [inconshreveable/log15](https://github.com/inconshreveable/log15), simple, powerful logging for Go ★
- [mailgun/vulcand](https://github.com/vulcand/vulcand), programmatic load balancer backed by etcd
- [mattheath/phosphor](https://github.com/mondough/phosphor), distributed system tracing
- [pivotal-golang/lager](https://github.com/pivotal-golang/lager), an opinionated logging library
- [rubyist/circuitbreaker](https://github.com/rubyist/circuitbreaker), circuit breaker library
- [sirupsen/logrus](https://github.com/sirupsen/logrus), structured, pluggable logging for Go ★
- [sourcegraph/appdash](https://github.com/sourcegraph/appdash), application tracing system based on Google's Dapper
- [spacemonkeygo/monitor](https://github.com/spacemonkeygo/monitor), data collection, monitoring, instrumentation, and Zipkin client library
- [streadway/handy](https://github.com/streadway/handy), net/http handler filters
- [vitess/rpcplus](https://godoc.org/github.com/youtube/vitess/go/rpcplus), package rpc + context.Context
- [gdamore/mangos](https://github.com/gdamore/mangos), nanomsg implementation in pure Go

### Web frameworks

- [Gorilla](http://www.gorillatoolkit.org)
- [Gin](https://gin-gonic.com/)
- [Negroni](https://github.com/codegangsta/negroni)
- [Goji](https://github.com/zenazn/goji)
- [Martini](https://github.com/go-martini/martini)
- [Beego](https://beego.vip/)
- [Revel](https://revel.github.io/) (considered [harmful](https://github.com/go-kit/kit/issues/350))
- [GoBuffalo](https://gobuffalo.io/)

## Additional reading

- [Architecting for the Cloud](https://slideshare.net/stonse/architecting-for-the-cloud-using-netflixoss-codemash-workshop-29852233) — Netflix
- [Dapper, a Large-Scale Distributed Systems Tracing Infrastructure](http://research.google.com/pubs/pub36356.html) — Google
- [Your Server as a Function](http://monkey.org/~marius/funsrv.pdf) (PDF) — Twitter
