---
layout: default
title: gRPC-Gateway
nav_order: 0
description: "Documentation site for the gRPC-Gateway"
permalink: /
---

# gRPC-Gateway
{: .fs-9 }

gRPC-Gateway is a plugin of [protoc](https://github.com/protocolbuffers/protobuf). It reads a [gRPC](https://grpc.io/) service definition and generates a reverse-proxy server which translates a RESTful JSON API into gRPC. This server is generated according to [custom options](https://cloud.google.com/service-infrastructure/docs/service-management/reference/rpc/google.api#http) in your gRPC definition.
{: .fs-6 .fw-300 }

[Get started](#getting-started){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 } [View it on GitHub](https://github.com/grpc-ecosystem/grpc-gateway){: .btn .fs-5 .mb-4 .mb-md-0 }

---

## Getting started

<a href="https://github.com/grpc-ecosystem/grpc-gateway/actions/workflows/main.yml"><img src="https://img.shields.io/github/workflow/status/grpc-ecosystem/grpc-gateway/main?color=379c9c&label=build&logo=github&logoColor=ffffff&style=flat-square"/></a>
<a href="https://app.slack.com/client/T029RQSE6/CBATURP1D"><img src="https://img.shields.io/badge/slack-grpc--gateway-379c9c?logo=slack&logoColor=ffffff&style=flat-square"/></a>
<a href="https://github.com/grpc-ecosystem/grpc-gateway/blob/main/LICENSE.txt"><img src="https://img.shields.io/github/license/grpc-ecosystem/grpc-gateway?color=379c9c&style=flat-square"/></a>
<a href="https://github.com/grpc-ecosystem/grpc-gateway/releases"><img src="https://img.shields.io/github/v/release/grpc-ecosystem/grpc-gateway?color=379c9c&logoColor=ffffff&style=flat-square"/></a>
<a href="https://github.com/grpc-ecosystem/grpc-gateway/stargazers"><img src="https://img.shields.io/github/stars/grpc-ecosystem/grpc-gateway?color=379c9c&style=flat-square"/></a>

gRPC-Gateway helps you to provide your APIs in both gRPC and RESTful style at the same time.

<div align="center">
<img src="assets/images/architecture_introduction_diagram.svg" />
</div>

To learn more about gRPC-Gateway check out the documentation.

## Contribution

See [CONTRIBUTING.md](https://github.com/grpc-ecosystem/grpc-gateway/blob/main/CONTRIBUTING.md).

## License

gRPC-Gateway is licensed under the BSD 3-Clause License.

See [LICENSE.txt](https://github.com/grpc-ecosystem/grpc-gateway/blob/main/LICENSE.txt) for more details.

### Thank you to the contributors of gRPC-Gateway

<ul class="list-style-none">
{% for contributor in site.github.contributors %}
<li class="d-inline-block mr-1">
<a href="{{ contributor.html_url }}"><img src="{{ contributor.avatar_url }}" width="32" height="32" alt="{{ contributor.login }}"/></a>
</li>
{% endfor %}
</ul>
