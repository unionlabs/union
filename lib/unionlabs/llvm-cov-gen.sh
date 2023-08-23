/nix/store/nnn0c6kzjvr6r8x68lx3087qqxq7qyxi-rust-default-1.71.0-nightly-2023-05-16/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-cov \
  show \
  -Xdemangler=/home/ben/.cargo/bin/rustfilt \
  \
  --object target/x86_64-unknown-linux-gnu/coverage/x86_64-unknown-linux-gnu/release/duration_from_str \
  --instr-profile=fuzz/coverage/duration_from_str/coverage.profdata \
  \
  --object target/x86_64-unknown-linux-gnu/coverage/x86_64-unknown-linux-gnu/release/duration \
  --instr-profile=fuzz/coverage/duration/coverage.profdata \
  \
  --object target/x86_64-unknown-linux-gnu/coverage/x86_64-unknown-linux-gnu/release/duration_checked_add \
  --instr-profile=fuzz/coverage/duration_checked_add/coverage.profdata \
  --show-line-counts-or-regions \
  --show-instantiations \
  --show-branches percent \
  --sources /home/ben/projects/union/union \
  --format html \
  --output-dir="./cov"
