# Dev scripts
For manual testing. Works on my box(*) ...


*) OSX

```
make install
cd scripts/contrib/local
rm -rf /tmp/trash
HOME=/tmp/trash bash setup_wasmd.sh
HOME=/tmp/trash bash start_node.sh
```

Next shell:

```
cd scripts/contrib/local
HOME=/tmp/trash bash 01-accounts.sh
HOME=/tmp/trash bash 02-contracts.sh
```

## Shell script development

[Use `shellcheck`](https://www.shellcheck.net/) to avoid common mistakes in shell scripts.
[Use `shfmt`](https://github.com/mvdan/sh) to ensure a consistent code formatting.
