import type { PageLoad } from "./$types.ts"

export const load: PageLoad = ({ params }) => {
  return {
    packetHash: params.packet_hash
  }
}
