import {Effect} from "effect";
import {Channel, Channels} from "$lib/schema/channel.ts";
import {ChannelValidationError} from "$lib/services/transfer-ucs03-evm/errors.ts";

export const getChannelInfoEffect = (
  source_chain_id: string,
  destination_chain_id: string,
  channels: typeof Channels.Type
): Effect.Effect<typeof Channel.Type, ChannelValidationError> =>
  Effect.gen(function* () {
    const rawChannel = channels.find(
      chan =>
        chan.source_chain_id === source_chain_id && chan.destination_chain_id === destination_chain_id
    )

    if (
      !rawChannel ||
      rawChannel.source_connection_id === null ||
      rawChannel.source_channel_id === null ||
      !rawChannel.source_port_id ||
      rawChannel.destination_connection_id === null ||
      rawChannel.destination_channel_id === null ||
      !rawChannel.destination_port_id
    ) {
      return yield* Effect.fail(new ChannelValidationError({
        source_chain_id,
        destination_chain_id,
        cause: 'Missing required channel information'
      }))
    }

    return yield* Effect.try({
      try: () => {
        let source_port_id = String(rawChannel.source_port_id)
        if (source_port_id.length < 4) {
          throw new Error('source_port_id is too short')
        }
        source_port_id = source_port_id.slice(2)

        let destination_port_id = String(rawChannel.destination_port_id)
        if (destination_port_id.length < 4) {
          throw new Error('destination_port_id is too short')
        }
        destination_port_id = destination_port_id.slice(2)

        return new Channel({
          source_chain_id,
          source_connection_id: rawChannel.source_connection_id!,
          source_channel_id: rawChannel.source_channel_id!,
          source_port_id,
          destination_chain_id,
          destination_connection_id: rawChannel.destination_connection_id!,
          destination_channel_id: rawChannel.destination_channel_id!,
          destination_port_id
        })
      },
      catch: err => new ChannelValidationError({
        source_chain_id,
        destination_chain_id,
        cause: err
      })
    })
  })

// Safe synchronous wrapper function (similar to getDerivedReceiverSafe in your example)
export const getChannelInfoSafe = (
  source_chain_id: string,
  destination_chain_id: string,
  channels: typeof Channels.Type
): typeof Channel.Type | null => {
  const result = Effect.runSync(
    Effect.either(getChannelInfoEffect(source_chain_id, destination_chain_id, channels))
  )

  return result._tag === 'Right' ? result.right : null
}