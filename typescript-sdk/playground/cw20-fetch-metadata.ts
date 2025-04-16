import { queryCosmosC20TokenMetadata } from "../src/mod.ts"

const metadata = await queryCosmosC20TokenMetadata({
  contractAddress: "bbn192gwgengt32um4qshvhg5f3prtey2g6xmyrmkd87xvy59elaw5vs504zeg",
  chainId: "bbn-test-5"
})

if (metadata.isErr()) {
  console.error(metadata.error)
} else {
  console.info(metadata.value)
}
