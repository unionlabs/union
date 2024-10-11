# What is an SDK standard?

An SDK standard is a design document describing a particular protocol, standard, or feature expected to be used by the Cosmos SDK. A SDK standard should list the desired properties of the standard, explain the design rationale, and provide a concise but comprehensive technical specification. The primary author is responsible for pushing the proposal through the standardization process, soliciting input and support from the community, and communicating with relevant stakeholders to ensure (social) consensus.

## Sections

A SDK standard consists of:

* a synopsis, 
* overview and basic concepts,
* technical specification,
* history log, and
* copyright notice.

All top-level sections are required. References should be included inline as links, or tabulated at the bottom of the section if necessary.  Included sub-sections should be listed in the order specified below. 

### Table Of Contents 
 
Provide a table of contents at the top of the file to assist readers.

### Synopsis

The document should include a brief (~200 word) synopsis providing a high-level description of and rationale for the specification.

### Overview and basic concepts

This section should include a motivation sub-section and a definitions sub-section if required:

* *Motivation* - A rationale for the existence of the proposed feature, or the proposed changes to an existing feature.
* *Definitions* - A list of new terms or concepts utilized in the document or required to understand it.

### System model and properties

This section should include an assumptions sub-section if any, the mandatory properties sub-section, and a dependencies sub-section. Note that the first two sub-section are tightly coupled: how to enforce a property will depend directly on the assumptions made. This sub-section is important to capture the interactions of the specified feature with the "rest-of-the-world", i.e., with other features of the ecosystem.

* *Assumptions* - A list of any assumptions made by the feature designer. It should capture which features are used by the feature under specification, and what do we expect from them.
* *Properties* - A list of the desired properties or characteristics of the feature specified, and expected effects or failures when the properties are violated. In case it is relevant, it can also include a list of properties that the feature does not guarantee.
* *Dependencies* - A list of the features that use the feature under specification and how.

### Technical specification

This is the main section of the document, and should contain protocol documentation, design rationale, required references, and technical details where appropriate.
The section may have any or all of the following sub-sections, as appropriate to the particular specification. The API sub-section is especially encouraged when appropriate.

* *API* - A detailed description of the feature's API.
* *Technical Details* - All technical details including syntax, diagrams, semantics, protocols, data structures, algorithms, and pseudocode as appropriate. The technical specification should be detailed enough such that separate correct implementations of the specification without knowledge of each other are compatible.
* *Backwards Compatibility* - A discussion of compatibility (or lack thereof) with previous feature or protocol versions.
* *Known Issues* - A list of known issues. This sub-section is specially important for specifications of already in-use features.
* *Example Implementation* - A concrete example implementation or description of an expected implementation to serve as the primary reference for implementers.

### History

A specification should include a history section, listing any inspiring documents and a plaintext log of significant changes.

See an example history section [below](#history-1).

### Copyright

A specification should include a copyright section waiving rights via [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0).

## Formatting

### General

Specifications must be written in GitHub-flavoured Markdown.

For a GitHub-flavoured Markdown cheat sheet, see [here](https://github.com/adam-p/markdown-here/wiki/Markdown-Cheatsheet). For a local Markdown renderer, see [here](https://github.com/joeyespo/grip).

### Language

Specifications should be written in Simple English, avoiding obscure terminology and unnecessary jargon. For excellent examples of Simple English, please see the [Simple English Wikipedia](https://simple.wikipedia.org/wiki/Main_Page).

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in specifications are to be interpreted as described in [RFC 2119](https://tools.ietf.org/html/rfc2119).

### Pseudocode

Pseudocode in specifications should be language-agnostic and formatted in a simple imperative standard, with line numbers, variables, simple conditional blocks, for loops, and
English fragments where necessary to explain further functionality such as scheduling timeouts. LaTeX images should be avoided because they are difficult to review in diff form.

Pseudocode for structs can be written in a simple language like Typescript or golang, as interfaces.

Example Golang pseudocode struct:

```go
type CacheKVStore interface {
  cache: map[Key]Value
  parent: KVStore
  deleted: Key
}
```

Pseudocode for algorithms should be written in simple Golang, as functions.

Example pseudocode algorithm:

```go
func get(
  store CacheKVStore,
  key Key) Value {

  value = store.cache.get(Key)
  if (value !== null) {
    return value
  } else {
    value = store.parent.get(key)
    store.cache.set(key, value)
    return value
  }
}
```

## History

This specification was significantly inspired by and derived from IBC's [ICS](https://github.com/cosmos/ibc/blob/main/spec/ics-001-ics-standard/README.md), which
was in turn derived from Ethereum's [EIP 1](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1.md).

Nov 24, 2022 - Initial draft finished and submitted as a PR

## Copyright

All content herein is licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0).
