import { queryCosmosCW20AddressBalance } from "../src/mod.ts"

const balance = await queryCosmosCW20AddressBalance({
  address: "bbn1xe0rnlh3u05qkwytkwmyzl86a0mvpwfxgf2t7u",
  contractAddress: "bbn192gwgengt32um4qshvhg5f3prtey2g6xmyrmkd87xvy59elaw5vs504zeg",
  chainId: "bbn-test-5"
})

if (balance.isErr()) {
  console.error(balance.error)
} else {
  console.info(balance.value)
}
