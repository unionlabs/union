import { redirect } from "@sveltejs/kit"
import { PacketHash } from "@unionlabs/sdk/schema"
import * as E from "effect/Either"
import * as S from "effect/Schema"
import type { PageLoad } from "./$types"

export const load: PageLoad = ({ params }) => {
  const packetHash = S.decodeEither(PacketHash)(params.packet_hash)

  return E.match(packetHash, {
    onLeft: () => {
      throw redirect(302, "/explorer/transfers")
    },
    onRight: (packetHash) => ({
      packetHash,
    }),
  })
}
