# Frequently Asked Questions

As we continue onboarding validators and expanding the testnet, we'll populate this page with FAQs in regard to becoming a validator on the Union Testnet.

## Questions & Answers

### How can I confirm my position as a testnet validator?

We started our testnet with the goal of onboarding 64 validators. The sign-up form for these first 64 slots has been closed. We later plan to expand to 128 testnet validators. To stay up to date with the most recent news on Union and our first public testnet, please [join our Discord](https://discord.gg/union-build).

### Do you have any public REST/RPC/GRPC endpoints?

Yes, you can find them listed on the [Public Endpoints](./public_endpoints) page.

### Why can't I submit transactions to `localhost` when using `docker run`?

Ensure you have exposed your host machine's network via the `--network` flag in docker. See our [`docker run uniond` alias](./obtaining_uniond#issuing-sub-commands-to-uniond) for more information.
