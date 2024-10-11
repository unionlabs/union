# ADR 002: SDK Documentation Structure

## Context

There is a need for a scalable structure of the Cosmos SDK documentation. Current documentation includes a lot of non-related Cosmos SDK material, is difficult to maintain and hard to follow as a user.

Ideally, we would have:

* All docs related to dev frameworks or tools live in their respective github repos (sdk repo would contain sdk docs, hub repo would contain hub docs, lotion repo would contain lotion docs, etc.)
* All other docs (faqs, whitepaper, high-level material about Cosmos) would live on the website.

## Decision

Re-structure the `/docs` folder of the Cosmos SDK github repo as follows:

```text
docs/
├── README
├── intro/
├── concepts/
│   ├── baseapp
│   ├── types
│   ├── store
│   ├── server
│   ├── modules/
│   │   ├── keeper
│   │   ├── handler
│   │   ├── cli
│   ├── gas
│   └── commands
├── clients/
│   ├── lite/
│   ├── service-providers
├── modules/
├── spec/
├── translations/
└── architecture/
```

The files in each sub-folders do not matter and will likely change. What matters is the sectioning:

* `README`: Landing page of the docs.
* `intro`: Introductory material. Goal is to have a short explainer of the Cosmos SDK and then channel people to the resources they need. The [Cosmos SDK tutorial](https://github.com/cosmos/sdk-application-tutorial/) will be highlighted, as well as the `godocs`.
* `concepts`: Contains high-level explanations of the abstractions of the Cosmos SDK. It does not contain specific code implementation and does not need to be updated often. **It is not an API specification of the interfaces**. API spec is the `godoc`.
* `clients`: Contains specs and info about the various Cosmos SDK clients.
* `spec`: Contains specs of modules, and others.
* `modules`: Contains links to `godocs` and the spec of the modules.
* `architecture`: Contains architecture-related docs like the present one.
* `translations`: Contains different translations of the documentation.

Website docs sidebar will only include the following sections:

* `README`
* `intro`
* `concepts`
* `clients`

`architecture` need not be displayed on the website.

## Status

Accepted

## Consequences

### Positive

* Much clearer organisation of the Cosmos SDK docs.
* The `/docs` folder now only contains Cosmos SDK and gaia related material. Later, it will only contain Cosmos SDK related material.
* Developers only have to update `/docs` folder when they open a PR (and not `/examples` for example).
* Easier for developers to find what they need to update in the docs thanks to reworked architecture.
* Cleaner `vuepress` build for website docs.
* Will help build an executable doc (cf https://github.com/cosmos/cosmos-sdk/issues/2611)

### Neutral

* We need to move a bunch of deprecated stuff to `/_attic` folder.
* We need to integrate content in `docs/sdk/docs/core` in `concepts`.
* We need to move all the content that currently lives in `docs` and does not fit in new structure (like `lotion`, intro material, whitepaper) to the website repository.
* Update `DOCS_README.md`

## References

* https://github.com/cosmos/cosmos-sdk/issues/1460
* https://github.com/cosmos/cosmos-sdk/pull/2695
* https://github.com/cosmos/cosmos-sdk/issues/2611
