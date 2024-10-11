run-clang-tidy -fix -extra-arg=-std=c++11 -extra-arg=-Wno-unknown-warning-option `git diff --name-only origin/HEAD`
