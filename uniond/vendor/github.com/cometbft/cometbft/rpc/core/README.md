# CometBFT RPC

## Pagination

Requests that return multiple items will be paginated to 30 items by default.
You can specify further pages with the ?page parameter. You can also set a
custom page size up to 100 with the ?per_page parameter.

## Subscribing to events

The user can subscribe to events emitted by CometBFT, using `/subscribe`. If
the maximum number of clients is reached or the client has too many
subscriptions, an error will be returned. The subscription timeout is 5 sec.
Each subscription has a buffer to accommodate short bursts of events or some
slowness in clients. If the buffer gets full, the subscription will be canceled
("client is not pulling messages fast enough"). If CometBFT exits, all
subscriptions are canceled ("CometBFT exited"). The user can unsubscribe
using either `/unsubscribe` or `/unsubscribe_all`.
