# Docgen

Docgen is a shell script for generating documentation files such as README.md's that are templated. Simply run

```
nix run .\#docgen
```

from the root of the repository to generate all documentation.

Docgen also defines a check which verifies that no files were altered while generating docs, which we use in CI to ensure we do not forget to generate documentation.
