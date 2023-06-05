# Contributing

Currently docgen is quite tightly integrated with the docs for each project. Ideally we'd move to a model where each flake-part exports a function/derivation to generate it's docs, and docgen acting like a registrar and runner. This would keep each flake-part modular. For now, when adding new docs to be generated, edit docgen.
