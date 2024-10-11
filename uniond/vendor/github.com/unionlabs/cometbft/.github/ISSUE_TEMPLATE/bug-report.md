---
name: Bug report
about: Create a report to help us squash bugs!
labels: bug, needs-triage
---
<!--

Please fill in as much of the template below as you can.

If you have general questions, please create a new discussion:
https://github.com/cometbft/cometbft/discussions

Be ready for followup questions, and please respond in a timely manner. We might
ask you to provide additional logs and data (CometBFT & App).

-->

## Bug Report

### Setup

**CometBFT version** (use `cometbft version` or `git rev-parse --verify HEAD` if installed from source):

**Have you tried the latest version**: yes/no

**ABCI app** (name for built-in, URL for self-written if it's publicly available):

**Environment**:
- **OS** (e.g. from /etc/os-release):
- **Install tools**:
- **Others**:

**node command runtime flags**:

### Config

<!--

You can paste only the changes you've made.

-->

### What happened?

### What did you expect to happen?

### How to reproduce it

<!--

Provide a description here as minimally and precisely as possible as to how to
reproduce the issue. Ideally only using our kvstore application, as debugging
app chains is not within our team's scope.

-->

### Logs

<!--

Paste a small part showing an error (< 10 lines) or link a pastebin, gist, etc.
containing more of the log file).

-->

### `dump_consensus_state` output

<!--

Please provide the output from the `http://<ip>:<port>/dump_consensus_state` RPC
endpoint for consensus bugs.

-->

### Anything else we need to know

<!--

Is there any additional information not covered by the other sections that would
help us to triage/debug/fix this issue?

-->

