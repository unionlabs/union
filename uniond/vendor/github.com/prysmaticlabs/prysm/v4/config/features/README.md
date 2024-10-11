# Prysm Feature Flags

Part of Prysm's feature development often involves use of feature flags which serve as a way to
toggle new features as they are introduced. Using this methodology, you are assured that your
feature can be safely tested in production with a fall back option if any regression were to occur.
This reduces the likelihood of an emergency release or rollback of a given feature due to
unforeseen issues.

## When to use a feature flag?

It is best to use a feature flag any time you are adding or removing functionality in an existing
critical application path. 

Examples of when to use a feature flag:

- Adding a caching layer to an expensive RPC call.
- Optimizing a particular algorithm or routine.
- Introducing a temporary public testnet configuration.

Examples of when not to use a feature flag:

- Adding a new gRPC endpoint. (Unless dangerous or expensive to expose).
- Adding new logging statements.
- Removing dead code.
- Adding any trivial feature with no risk of regressions to existing functionality.

## How to use a feature flag?

Once it has been decided that you should use a feature flag. Follow these steps to safely
releasing your feature. In general, try to create a single PR for each step of this process.

1. Add your feature flag to `shared/featureconfig/flags.go`, use the flag to toggle a boolean in the
feature config in shared/featureconfig/config.go. It is a good idea to use the `enable` prefix for
your flag since you're going to invert the flag in a later step. i.e you will use `disable` prefix
later. For example, `--enable-my-feature`. Additionally, [create a feature flag tracking issue](https://github.com/prysmaticlabs/prysm/issues/new?template=feature_flag.md) 
for your feature using the appropriate issue template.
2. Use the feature throughout the application to enable your new functionality and be sure to write
tests carefully and thoughtfully to ensure you have tested all of your new functionality without losing
coverage on the existing functionality. This is considered an opt-in feature flag. Example usage:
```go
func someExistingMethod(ctx context.Context) error {
    if features.Get().MyNewFeature {
       return newMethod(ctx)
    }
    // Otherwise continue with the existing code path.
}
``` 
3. Add the flag to the end to end tests. This set of flags can also be found in shared/featureconfig/flags.go. 
4. Test the functionality locally and safely in production. Once you have enough confidence that
your new function works and is safe to release then move onto the next step.
5. Move your existing flag to the deprecated section of `shared/featureconfig/flags.go`. It is
important NOT to delete your existing flag outright. Deleting a flag can be extremely frustrating
to users as it may break their existing workflow! Marking a flag as deprecated gives users time to
adjust their start scripts and workflow. Add another feature flag to represent the inverse of your
flag from step 1. For example `--disable-my-feature`. Read the value of this flag to appropriately
the config value in shared/featureconfig/config.go.
6. After your feature has been included in a release as "opt-out" and there are no issues,
deprecate the opt-out feature flag, delete the config field from shared/featureconfig/config.go,
delete any deprecated / obsolete code paths.

Deprecated flags are deleted upon each major semver point release. Ex: v1, v2, v3.
