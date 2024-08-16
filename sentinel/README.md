# Sentinel

Sentinels are e2e tests which can continiously run against production environments to ensure that 
Union is fully functional. 

## Usage

Each sentinel has a sane default configuration, as well as that their configuration can be (partially) overridden by
passing a JSON object to the run command

```bash
cargo run -- run --overrides '{"check_ibc_trace_latency": {"limit": 10}}' # change the limit config of check_ibc_trace_latency, but leave everything else identical.
```

Individual tests can be run by passing a regex to filter, which cause only the matches to be run.