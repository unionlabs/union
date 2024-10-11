# Format Guidelines

If you are interesting in contributing to the flatbuffers project, please take a second to read this document. Each language has it's own set of rules, that are defined in their respective formatter/linter documents.

# Notes

- Run the linter on the language you are working on before making a Pull Request.
- DONT format/lint the generated code.

# Languages

## C++

C++ uses `clang-format` as it's formatter. Run the following script `sh scripts/clang-format-git.sh`, and it should style the C++ code according to [google style guide](https://google.github.io/styleguide/cppguide.html).

## Swift

Swift uses swiftformat as it's formatter. Take a look at [how to install here](https://github.com/nicklockwood/SwiftFormat/blob/master/README.md#how-do-i-install-it). Run the following command `swiftformat --config swift.swiftformat .` in the root directory of the project

## Typescript

Typescript uses eslint as it's linter. Take a look at [how to install here](https://eslint.org/docs/user-guide/getting-started). Run the following command `eslint ts/** --ext .ts` in the root directory of the project