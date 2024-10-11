# Cosmos SDK Core

The [cosmossdk.io/core](https://pkg.go.dev/cosmossdk.io/core) Go module defines essential APIs and interfaces for the Cosmos SDK ecosystem. It serves as a foundation for building modular blockchain applications.

Key features and principles:

1. Provides stable, long-term maintained APIs for module development and app composition.
2. Focuses on interface definitions without implementation details.
3. Implementations are housed in the runtime(/v2) or individual modules.
4. Modules depend solely on core APIs for maximum compatibility.
5. New API additions undergo thorough consideration to maintain stability.
6. Adheres to a no-breaking-changes policy for reliable dependency management.
7. Aimed to have zero dependencies, ensuring a lightweight and self-contained foundation.

The core module offers the [appmodule](https://pkg.go.dev/cosmossdk.io/core/appmodule) and [appmodule/v2](https://pkg.go.dev/cosmossdk.io/core/appmodule/v2) packages that include APIs to describe how modules can be written.
Additionally, it contains all core services APIs that can be used in modules to interact with the SDK, majoritarily via the `appmodule.Environment` struct.
Last but not least, it provides codecs and packages for the Cosmos SDK's core types (think of, for instance, logger, store interface or an address codec).

Developers and contributors approach core API design with careful deliberation, ensuring that additions provide significant value while maintaining the module's stability and simplicity.
