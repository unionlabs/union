# Voyager Transaction Batch Plugin

This plugin provides simple, naive event handling and transaction batching for both IBC v1 (IBC classic) and IBC union, by producing the correct "response" messages to a given IBC event (i.e., `ConnectionOpenInit` -> `MsgConnectionOpenTry`)  and batching them with as few client updates as possible.

For example, given the config:

```json
{
	"chain_id": "32382",
	"client_configs": [
		{
			"client_id": 1,
			"min_batch_size": 1,
			"max_batch_size": 3,
			"max_wait_time": {
				"secs": 10,
				"nanos": 0
			}
		}
	]
}
```

For all events on chain `32382` that are related to client `1`, events will be batched with the following logic:

- the batch size will be `min_batch_size..=max_batch_size` (inclusive)
- if `max_batch_size` is not hit (if there aren't enough events), then messages will be held for no longer than `max_wait_time`. "Overdue" message batches skip directly to [client updates](#client-updates).
- messages are sorted by their age within batches:
  ```
  [[1, 2, 3], [4, 5, 6]]
  ```

## Client Updates

Given a group of message batches, a client update will be generated for the max provable height of all batches, allowing for all of the messages in the batches to use one client update. Additionally, additional checks are performed to ensure that the client update is actually required, avoiding potentially expensive client update transactions.
