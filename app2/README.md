# Union App V2w

## Developing

```bash
nix run .#app2-dev-server -L
```

## Building

To create a production version of your app:

```bash
nix build .#app2 -L
```

## Component naming conventions

We use hypenated components

## Resource Naming Conventions

Imagine you have a resource `Block` that you fetch from the api, want to store globally, and have a component that displays it nicely. The naming would be as follows:

- The `Schema` is called `Block` and stored in `src/lib/schemas/block.ts`
- The `Store` class is called `BlockStore` and stored in `src/lib/stores/block.svelte.ts`
- The concrete instance of `BlockStore` is called `block`
- The coponent displaying blocks is called `BlockComponent` and stored in `src/lib/components/data/block-component.svelte`

## UI Components Naming Conventions

A `Button` would go in `src/lib/components/ui/button/index.svelte`
