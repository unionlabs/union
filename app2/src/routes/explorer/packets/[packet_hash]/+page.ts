import type { PageLoad } from './$types';

export const load: PageLoad = ({ params }) => {
  return {
    packetHash: params.packet_hash
  };
};
