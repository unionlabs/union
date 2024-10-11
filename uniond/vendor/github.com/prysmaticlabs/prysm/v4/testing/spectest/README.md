# Spec Tests

Spec testing vectors: https://github.com/ethereum/consensus-spec-tests

To run all `mainnet` spec tests:

```bash
bazel test //... --test_tag_filters=spectest
```

Minimal tests require `--define ssz=minimal` setting and are not triggered
automatically when `//...` is selected. One can run minimal tests manually, though:

```bash
bazel query 'tests(attr("tags", "minimal, spectest", //...))' | xargs bazel test --define ssz=minimal
```
