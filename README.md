[![build](https://github.com/UnionFi/union/actions/workflows/main.yml/badge.svg)](https://github.com/UnionFi/union/actions/workflows/main.yml)

# Union

This repository hosts the [`uniond`](./uniond/) codebase, the node for the Union network.

## Building

We make use of the [`nix`](https://nixos.org/) build system. To obtain a binary of `uniond`, run:

```bash
nix build .\#uniond
```
