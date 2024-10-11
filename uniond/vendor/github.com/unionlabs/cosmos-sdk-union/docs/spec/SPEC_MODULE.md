# Specification of Modules

This file intends to outline the common structure for specifications within
this directory.

## Tense

For consistency, specs should be written in passive present tense.

## Pseudo-Code

Generally, pseudo-code should be minimized throughout the spec. Often, simple
bulleted-lists which describe a function's operations are sufficient and should
be considered preferable. In certain instances, due to the complex nature of
the functionality being described pseudo-code may the most suitable form of
specification. In these cases use of pseudo-code is permissible, but should be
presented in a concise manner, ideally restricted to only the complex
element as a part of a larger description.

## Common Layout

The following generalized `README` structure should be used to breakdown
specifications for modules. The following list is nonbinding and all sections are optional.

* `# {Module Name}` - overview of the module
* `## Concepts` - describe specialized concepts and definitions used throughout the spec
* `## State` - specify and describe structures expected to be marshalled into the store, and their keys
* `## State Transitions` - standard state transition operations triggered by hooks, messages, etc.
* `## Messages` - specify message structure(s) and expected state machine behaviour(s)
* `## Begin Block` - specify any begin-block operations
* `## End Block` - specify any end-block operations
* `## Hooks` - describe available hooks to be called by/from this module
* `## Events` - list and describe event tags used
* `## Client` - list and describe CLI commands and gRPC and REST endpoints
* `## Params` - list all module parameters, their types (in JSON) and examples
* `## Future Improvements` - describe future improvements of this module
* `## Tests` - acceptance tests
* `## Appendix` - supplementary details referenced elsewhere within the spec

### Notation for key-value mapping

Within `## State` the following notation `->` should be used to describe key to
value mapping:

```text
key -> value
```

to represent byte concatenation the `|` may be used. In addition, encoding
type may be specified, for example:

```text
0x00 | addressBytes | address2Bytes -> amino(value_object)
```

Additionally, index mappings may be specified by mapping to the `nil` value, for example:

```text
0x01 | address2Bytes | addressBytes -> nil
```
