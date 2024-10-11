---
category: documentation
name: 2020 Season of Docs
---

# 2020 Season of Docs

![Google Season of Docs](https://developers.google.com/season-of-docs/images/logo/SeasonofDocs_Logo_SecondaryGrey_300ppi.png "Season of Docs")

This year the gRPC-Gateway is participating in the [Google Season of Docs](https://g.co/seasonofdocs).
We're excited to see what contributions this will bring to our documentation.

## Project details

  - Organization name: **gRPC-Gateway**
  - Organization description: The gRPC-Gateway brings the power and safety of designing APIs with Protobuf and gRPC to the JSON/HTTP API world. It has several
    common use cases:
    - When a user wants to migrate an API to gRPC, but needs to expose a JSON/HTTP API
      to old users.
    - When a user wants to expose an existing gRPC API to a JSON/HTTP API audience.
    - When quickly iterating on a JSON/HTTP API design.
  - Website: https://grpc-ecosystem.github.io/grpc-gateway
  - Repo: https://github.com/grpc-ecosystem/grpc-gateway
  - Project administrators and mentors:
    - Johan Brandhorst (@johanbrandhorst)
    - Andrew Z Allen (@achew22)

## Project Ideas

### Refactor the existing docs site

Our existing docs site (this site!) is decidedly starting to look a bit dated. We'd love to
have a new version with some updated styling and a better structure. The existing content
could be preserved and just reused with a fresh new look, or we could rewrite much of it.

It's currently rendered from Markdown using [Jekyll](https://jekyllrb.com/).
[The source code](https://github.com/grpc-ecosystem/grpc-gateway/tree/master/docs)
for the site is part of the main repo.

We the best way to do this would be to have someone who is unfamiliar with the project
try to use the current material and note anything that was unclear and that they couldn't
easily find with our existing docs.

Material:
  - [The current site](https://grpc-ecosystem.github.io/grpc-gateway/)
  - [Jekyll](https://jekyllrb.com/) which powers the site now.
  - [The source code](https://github.com/grpc-ecosystem/grpc-gateway/tree/master/docs) for the site today.
  - [The project README](https://github.com/grpc-ecosystem/grpc-gateway/blob/master/README.md) which
    contains an intro to the project.

### Rewrite the README with a better intro and examples

The README has evolved since the start of the project and could do with a rewrite from
first principles. The README is the first thing our prospective users see, and it should
quickly and concisely answer the most important questions for our users.

  - What problems can the gRPC-Gateway solve?
  - How do I use the gRPC-Gateway?
  - What does a complete example look like?
  - Where can I find more information about using it?
  - Where can I learn more about the technologies the gRPC-Gateway is built on?
  - How do I submit an issue report or get help?

Material:
  - [The current README](https://github.com/grpc-ecosystem/grpc-gateway/blob/master/README.md).

### Create a tutorial for the docs site

We'd like to be able to point to a tutorial for one of the common use cases of the project.
The ones mentioned in the project details are the primary use cases we advertise:

  - When a user wants to migrate an API to gRPC, but needs to expose a JSON/HTTP API
    to old users.
  - When a user wants to expose an existing gRPC API to a JSON/HTTP API audience.
  - When quickly iterating on an JSON/HTTP API design.

It could be a single or several blog posts on our docs site, or another site, like Medium.

### Improve the "customize your gateway" section of the docs

This is where we've collected a lot of the little tips we've developed with
users that don't quite fit in the main README or documentation. It would be great
to have a look over this and add detail where possible and generally structure it
a bit better. Maybe it could be rewritten as a FAQ that details solutions to common issues?

Material:
  - [The customize your gateway page](https://grpc-ecosystem.github.io/grpc-gateway/docs/customizingyourgateway.html)

### Improve the contributor's guide

This is currently split between
[CONTRIBUTING.md](https://github.com/grpc-ecosystem/grpc-gateway/blob/master/CONTRIBUTING.md)
and the [issue templates](https://github.com/grpc-ecosystem/grpc-gateway/tree/master/.github/ISSUE_TEMPLATE).
Both of these are a little ad-hoc and could do with a fresh pair of eyes.

Material:
  - [Current CONTRIBUTING.md](https://github.com/grpc-ecosystem/grpc-gateway/blob/master/CONTRIBUTING.md)
  - [Current issue templates](https://github.com/grpc-ecosystem/grpc-gateway/tree/master/.github/ISSUE_TEMPLATE)

### Write a v2.0.0 migration guide

We're planning on making a v2 release of the project, which will have some backwards-compatibility breaking changes.
We need to write a migration guide so that users know what to expect when upgrading their deployments.

This should include:

  - A list of all the breaking changes and their consequences for the user.
  - For each breaking change, a guide to how their systems may need to be changed.

Currently, the scope of the v2 release is not entirely known, as it is still in progress, but we will
endeavour not to make too many breaking changes as that will discourage users from upgrading.

Material:
  - [v2 Tracking issue](https://github.com/grpc-ecosystem/grpc-gateway/issues/1223)
