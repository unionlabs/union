# MOVEMENT IBC STACK NOTES

## Deploying IBC:

```
nix run .#movement move create-object-and-publish-package -- --address-name ibc --package-dir move/move-ibc --skip-fetch-latest-git-deps --override-size-check --included-artifacts none
```

Output of this will be something like:

```
Do you want to publish this package at object address 0x0662b7e00f8776697600f3356be2203b96d4a5f484cf8dde4e02cfd8e6e03de0 [yes/no] >
yes
package size 55530 bytes
Do you want to submit a transaction for a range of [4210100 - 6315100] Octas at a gas unit price of 100 Octas? [yes/no] >
yes
Transaction submitted: https://explorer.movementlabs.xyz/txn/0x7e0498c1a0c7c860f29530969f62276ceee09ecc5d66c010a749d2a302a78c8c?network=testnet
Code was successfully deployed to object address 0x0662b7e00f8776697600f3356be2203b96d4a5f484cf8dde4e02cfd8e6e03de0.
{
  "Result": "Success"
}
```

So the IBC address is `0x0662b7e00f8776697600f3356be2203b96d4a5f484cf8dde4e02cfd8e6e03de0`. Now to deploy zkgm:

## Deploying ZKGM :

```
nix run .#movement move create-object-and-publish-package --address-name zkgm --named-addresses ibc=0x0662b7e00f8776697600f3356be2203b96d4a5f484cf8dde4e02cfd8e6e03de0 --package-dir move/ucs03-zkgm --skip-fetch-latest-git-deps --override-size-check --included-artifacts none
```

Here the `ibc=` address should be defined with the address of the latest IBC address.

Output of this will be something like:

```
Do you want to publish this package at object address 0x0f81cb075f0a8536ba3069106c75c2476e0f904c95c8347b7c5b55b672cf220f [yes/no] >
yes
package size 27576 bytes
Do you want to submit a transaction for a range of [2678800 - 4018200] Octas at a gas unit price of 100 Octas? [yes/no] >
yes
Transaction submitted: https://explorer.movementlabs.xyz/txn/0x39228eaf3745908c2eb84eb575fbfb665f422931eb4e9a26e775736f79896bda?network=testnet
Code was successfully deployed to object address 0x0f81cb075f0a8536ba3069106c75c2476e0f904c95c8347b7c5b55b672cf220f.
{
  "Result": "Success"
}
```

## Upgrade IBC or ZKGM:

We can not change the interface, we can add new functions or the implementation of existing functions. For example if we don't want a specific function, we can abort in that but we can not delete it from the code.

We need to define `ibc` address under `move/move-ibc/Move.toml` like:

```
[addresses]
ibc = "0x0662b7e00f8776697600f3356be2203b96d4a5f484cf8dde4e02cfd8e6e03de0"
```

And we also need to specify that as object-address

```
nix run .#movement move upgrade-object-package -- --object-address 0x0662b7e00f8776697600f3356be2203b96d4a5f484cf8dde4e02cfd8e6e03de0 --package-dir move/move-ibc --skip-fetch-latest-git-deps --override-size-check --included-artifacts none
```

And the output of this will be:

```
Do you want to upgrade the package 'ibc' at object address 0x0662b7e00f8776697600f3356be2203b96d4a5f484cf8dde4e02cfd8e6e03de0 [yes/no] >
yes
package size 55638 bytes
Do you want to submit a transaction for a range of [19500 - 29200] Octas at a gas unit price of 100 Octas? [yes/no] >
yes
Transaction submitted: https://explorer.movementlabs.xyz/txn/0x64be7c0e194b42fbc8e4a564de71818b9ac9922e371fa1bf4a59e466f73739b8?network=testnet
Code was successfully upgraded at object address 0x0662b7e00f8776697600f3356be2203b96d4a5f484cf8dde4e02cfd8e6e03de0.
{
  "Result": "Success"
}
```

## Calling Function From Terminal

Example full query

```
nix run .#movement move run -- --function-id "0x0662b7e00f8776697600f3356be2203b96d4a5f484cf8dde4e02cfd8e6e03de0::ibc_app::transfer" --args u32:1 hex:"0x756E696F6E3136736A717330647565677268716E366732306D3278723474703678747630796D66753463756175616834346C793871666B7A6D71743879797778" address:"0xc72f198d6cc4a0ae4d7369fbf93cc60c56156548b051e5ccc9a27fb380099467" u256:1 hex:"0x6d756e6f" u256:1 u64:18446744073709551500 u64:18446744073709551500 hex:"0x53f247a39cb05a49ed206cdb7b09dad6a71b9eae2f49b3408be67510fd19b1ab"
```

### **Specifying the Function**

```bash
--function-id "0x0662b7e00f8776697600f3356be2203b96d4a5f484cf8dde4e02cfd8e6e03de0::ibc_app::transfer"
```

- **--function-id**: This option points to the fully qualified function being called.
- **0x0662b7e00f8776697600f3356be2203b96d4a5f484cf8dde4e02cfd8e6e03de0**: The object address where the module resides.
- **ibc_app**: The module inside the object that contains the function.
- **transfer**: The specific function being called.

______________________________________________________________________

### **Passing Function Arguments**

```bash
--args u32:1 hex:"0x756E696F6E3136736A717330647565677268716E366732306D3278723474703678747630796D66753463756175616834346C793871666B7A6D71743879797778" address:"0xc72f198d6cc4a0ae4d7369fbf93cc60c56156548b051e5ccc9a27fb380099467" u256:1 hex:"0x6d756e6f" u256:1 u64:18446744073709551500 u64:18446744073709551500 hex:"0x53f247a39cb05a49ed206cdb7b09dad6a71b9eae2f49b3408be67510fd19b1ab"
```

# Error namespaces

Since you can only have numeric errors in move, we split the errors to namespaces. Every other
N thousand specifies a new namespace:

- IBC: 35xxx
  - Cometbls light client: 351xx
  - State lens light client: 352xx
- UCS01 Relay: 36xxx
