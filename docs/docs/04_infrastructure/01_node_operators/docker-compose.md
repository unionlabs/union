---
title: "Docker Compose"
---

This guide assumes you are familiar with running a Union validator. If not, start with the [validator guide](./getting-started).

[docker-compose](https://docs.docker.com/compose/) is a tool for running containers in a declarative manner. This allows for better automation, upgrades, and monitoring.

## Configuration

Our base docker-compose.yml is fairly simple:

```yaml
version: "3.5"

services:
  node:
    image: ghcr.io/unionlabs/uniond:latest
    volumes: ~/.uniond:/root/.uniond
    ports:
      - "26657:26657"
      - "1317:1317"
      - "9093:9093"
    restart: unless-stopped
```

The only section of significance is the `volumes` key. Here we map an already initialized `~/.uniond` directory to the `node` service. The `~/.uniond` directory should contain a `config` and `data` directory. To properly set these up, check out the [validator guide](./getting-started#initialization).

## Monitoring

We suggest adding additional monitoring services, such as [datadog](https://www.datadoghq.com/).
