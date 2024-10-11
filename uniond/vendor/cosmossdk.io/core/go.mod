module cosmossdk.io/core

// Core is meant to have zero dependencies, so we can use it as a dependency
// in other modules without having to worry about circular dependencies.

go 1.23

// Version tagged too early and incompatible with v0.50 (latest at the time of tagging)
retract v0.12.0
