# Setting Up Continuous Integration with `wasm-bindgen-test`

This page contains example configurations for running `wasm-bindgen-test`-based
tests in various CI services.

Is your favorite CI service missing? [Send us a pull
request!](https://github.com/rustwasm/wasm-bindgen)

## Travis CI

```yaml
language: rust
rust    : nightly

addons:
  firefox: latest
  chrome : stable

install:
  - curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

script:

  # this will test the non wasm targets if your crate has those, otherwise remove this line.
  #
  - cargo test

  - wasm-pack test --firefox --headless
  - wasm-pack test --chrome  --headless
```

## AppVeyor

```yaml
install:
  - ps: Install-Product node 10
  - appveyor-retry appveyor DownloadFile https://win.rustup.rs/ -FileName rustup-init.exe
  - rustup-init.exe -y --default-host x86_64-pc-windows-msvc --default-toolchain nightly
  - set PATH=%PATH%;C:\Users\appveyor\.cargo\bin
  - rustc -V
  - cargo -V
  - rustup target add wasm32-unknown-unknown
  - cargo install wasm-bindgen-cli

build: false

test_script:
  # Test in Chrome. chromedriver is installed by default in appveyor.
  - set CHROMEDRIVER=C:\Tools\WebDriver\chromedriver.exe
  - cargo test --target wasm32-unknown-unknown
  - set CHROMEDRIVER=
  # Test in Firefox. geckodriver is also installed by default.
  - set GECKODRIVER=C:\Tools\WebDriver\geckodriver.exe
  - cargo test --target wasm32-unknown-unknown
```

## GitHub Actions

```yaml
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Install
        run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

      - run: cargo test
      - run: wasm-pack test --headless --chrome
      - run: wasm-pack test --headless --firefox
```
