These abis are made by running `nix build .#hubble-abis`, and then applying the following transformation in order to get viem typechecking:

1. Update file from .json to .ts format. For example, myAbi.json should be myAbi.ts,
1. Add export and const assertion

```diff
-[
+ export const myAbi = [
  {
    "inputs": [{ "name": "owner", "type": "address" }],
    "name": "balanceOf",
    "outputs": [{ name: "balance", type: "uint256" }],
    "stateMutability": "view",
    "type": "function",
  }
-]
+] as const
```

source: https://github.com/wevm/wagmi/discussions/1084#discussioncomment-3891592
