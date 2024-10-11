- `[flowrate]` Remove expensive time.Now() calls from flowrate calls.
  Changes clock updates to happen in a separate goroutine.
  ([\#3016](https://github.com/cometbft/cometbft/issues/3016))
