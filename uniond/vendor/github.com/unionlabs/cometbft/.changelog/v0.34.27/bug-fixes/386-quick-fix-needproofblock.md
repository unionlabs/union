- `[consensus]` Short-term fix for the case when `needProofBlock` cannot find
  previous block meta by defaulting to the creation of a new proof block.
  ([\#386](https://github.com/cometbft/cometbft/pull/386): @adizere)
  - Special thanks to the [Vega.xyz](https://vega.xyz/) team, and in particular
    to Zohar (@ze97286), for reporting the problem and working with us to get to
    a fix.
