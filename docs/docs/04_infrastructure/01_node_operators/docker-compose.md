---
title: "Docker Compose"
---

This guide assumes you are familiar with running a Union validator. If not, start with the [validator guide](./getting-started).

[docker-compose](https://docs.docker.com/compose/) is a tool for running containers in a declarative manner. This allows for better automation, upgrades, and monitoring.

## Configuration

Our base `compose.yml` is fairly simple:

```yaml
services:
  node:
    image: ghcr.io/unionlabs/uniond:${UNIOND_VERSION}
    volumes: 
        - ~/.union:/.union
    ports:
      - "26657:26657"
      - "26656:26656"
      - "1317:1317"
      - "9093:9093"
    restart: unless-stopped
    command: start --home /.union
```

Pay special attention to the `volumes` key. Here we map an already initialized `~/.uniond` directory to the `node` service. The `~/.uniond` directory should contain a `config` and `data` directory. To properly set these up, check out the [validator guide](./getting-started#initialization).

## Monitoring

We suggest adding additional monitoring services, such as [datadog](https://www.datadoghq.com/).
