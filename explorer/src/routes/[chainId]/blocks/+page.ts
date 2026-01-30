import { indexer, type IndexedBlock } from "$lib/services/indexer-client"
import { rest } from "$lib/services/chain-api"
import type { PageLoad } from "./$types"

// Convert indexed block to BlockSummary format for UI
function toBlockSummary(block: IndexedBlock) {
  return {
    height: String(block.height),
    hash: block.hash,
    time: block.time,
    proposer: block.proposer,
    txCount: block.tx_count,
    // Full data from indexer-v2
    header: block.header,
    signatures: block.signatures,
    txHashes: block.tx_hashes,
  }
}

export const load: PageLoad = async ({ depends, parent }) => {
  depends("blocks:recent")

  const { chainId, chain } = await parent()

  async function getBlocks() {
    const blocks = await indexer.blocks(chainId, 50)
    return blocks.map(toBlockSummary)
  }

  return {
    blocks: getBlocks(),
    validators: rest.validators(chainId, "BOND_STATUS_BONDED"),
    chain,
  }
}
