import { ChannelValidationError } from "$lib/services/transfer-ucs03-evm/errors"
import type { UniversalChainId } from "@unionlabs/sdk/schema"
import { Channel, type Channels } from "@unionlabs/sdk/schema"
import { Effect } from "effect"

// TODO(ehegnes): replace this with a schema transform
export const getChannelInfo = (
  source_universal_chain_id: UniversalChainId,
  destination_universal_chain_id: UniversalChainId,
  channels: typeof Channels.Type,
): Effect.Effect<typeof Channel.Type, ChannelValidationError> =>
  Effect.gen(function*() {
    const channel = channels.find(
      chan =>
        chan.destination_universal_chain_id === destination_universal_chain_id
        && chan.source_universal_chain_id === source_universal_chain_id,
    )

    if (
      !channel
      || channel.source_connection_id === null
      || channel.source_channel_id === null
      || !channel.source_port_id
      || channel.destination_connection_id === null
      || channel.destination_channel_id === null
      || !channel.destination_port_id
    ) {
      return yield* Effect.fail(
        new ChannelValidationError({
          source_universal_chain_id,
          destination_universal_chain_id,
          cause: "Missing required channel information",
        }),
      )
    }

    return new Channel({
      source_universal_chain_id,
      source_connection_id: channel.source_connection_id,
      source_channel_id: channel.source_channel_id,
      source_client_id: channel.source_client_id,
      source_port_id: channel.source_port_id,
      fees: channel.fees,
      destination_universal_chain_id,
      destination_connection_id: channel.destination_connection_id,
      destination_channel_id: channel.destination_channel_id,
      destination_client_id: channel.destination_client_id,
      destination_port_id: channel.destination_port_id,
      tags: channel.tags,
    })
  })
