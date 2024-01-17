<script lang="ts">

const versions = {
  "union-testnet-4": ["v0.14.0", "v0.15.0", "v0.16.0", "v0.17.0"],
  "union-testnet-5": ["v0.18.0"]
}

export let chainId = 'union-testnet-4'
export let genesis = false

function isKeyof<T extends object>(obj: T, possibleKey: keyof any): possibleKey is keyof T {
  return possibleKey in obj;
}

function Version(chainId: string, genesis: boolean) {
  if (isKeyof(versions, chainId)) {
    let version = genesis ? 0 : -1
    return versions[chainId].at(version)
  }
  throw new Error('Unknown chainId: ' + chainId)
}

$: _genesis = typeof genesis === 'string' ? JSON.parse(genesis) : genesis

</script>

{Version(chainId, _genesis)}
