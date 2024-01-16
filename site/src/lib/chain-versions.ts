import versions from "../../../versions.json"

function isKeyof<T extends object>(obj: T, possibleKey: keyof any): possibleKey is keyof T {
  return possibleKey in obj;
}

export default function Version(chainId: string, genesis: boolean) {
  if (isKeyof(versions, chainId)) {
    let version = genesis ? 0 : -1
    return versions[chainId].at(version)
  }
  throw new Error('Unknown chainId: ' + chainId)
}
