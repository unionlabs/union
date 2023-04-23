# Pruning

## Overview

Pruning is the mechanism for deleting old application heights from the disk. Depending on the use case,
nodes may require different pruning strategies. For example, archive nodes must keep all
the states and prune nothing. On the other hand, a regular validator node may want to only keep 100 latest heights for performance reasons.

## Strategies

The strategies are configured in `app.toml`, with the format `pruning = "<strategy>"` where the options are:

* `default`: only the last 362,880 states(approximately 3.5 weeks worth of state) are kept; pruning at 10 block intervals
* `nothing`: all historic states will be saved, nothing will be deleted (i.e. archiving node)
* `everything`: 2 latest states will be kept; pruning at 10 block intervals.
* `custom`: allow pruning options to be manually specified through 'pruning-keep-recent', and 'pruning-interval'

If no strategy is given to the BaseApp, `nothing` is selected. However, we perform validation on the CLI layer to require these to be always set in the config file.

## Custom Pruning

These are applied if and only if the pruning strategy is custom:

* `pruning-keep-recent`: N means to keep all of the last N states
* `pruning-interval`: N means to delete old states from disk every Nth block.

## Relationship to State Sync Snapshots

Snapshot settings are optional. However, if set, they have an effect on how pruning is done by
persisting the heights that are multiples of `state-sync.snapshot-interval` until after the snapshot is complete. See the "Relationship to Pruning" section in `snapshots/README.md` for more details.
