---
sidebar_position: 1
---

# Configuration

This documentation refers to the `app.toml`, if you'd like to learn more about CometBFT configuration (`config.toml`) please visit the [CometBFT Configuration Manual](https://docs.cometbft.com/v1.0/references/config/).

<!-- the following is not a python reference, however syntax coloring makes the file more readable in the docs -->
```python reference
https://github.com/cosmos/cosmos-sdk/blob/main/tools/confix/data/v0.47-app.toml 
```

## inter-block-cache

This feature will consume more ram than a normal node, if enabled.

## iavl-cache-size

Using this feature will increase ram consumption

## iavl-lazy-loading

This feature is to be used for archive nodes, allowing them to have a faster start up time. 
