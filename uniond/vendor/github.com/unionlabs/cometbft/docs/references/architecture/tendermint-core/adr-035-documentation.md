# ADR 035: Documentation

Author: @zramsay (Zach Ramsay)

## Changelog

###  November 2nd 2018

- initial write-up

## Context

The Tendermint documentation has undergone several changes until settling on the current model. Originally, the documentation was hosted on the website and had to be updated asynchronously from the code. Along with the other repositories requiring documentation, the whole stack moved to using Read The Docs to automatically generate, publish, and host the documentation. This, however, was insufficient; the RTD site had advertisement, it wasn't easily accessible to devs, didn't collect metrics, was another set of external links, etc.

## Decision

For two reasons, the decision was made to use VuePress:

1) ability to get metrics (implemented on both Tendermint and SDK)
2) host the documentation on the website as a `/docs` endpoint.

This is done while maintaining synchrony between the docs and code, i.e., the website is built whenever the docs are updated.

## Status

The two points above have been implemented; the `config.js` has a Google Analytics identifier and the documentation workflow has been up and running largely without problems for several months. Details about the documentation build & workflow can be found [here](../DOCS_README.md)

## Consequences

Because of the organizational separation between Tendermint & Cosmos, there is a challenge of "what goes where" for certain aspects of documentation.

### Positive

This architecture is largely positive relative to prior docs arrangements.

### Negative

A significant portion of the docs automation / build process is in private repos with limited access/visibility to devs. However, these tasks are handled by the SRE team.

### Neutral
