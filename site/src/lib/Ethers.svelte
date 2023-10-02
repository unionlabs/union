<script lang="ts">
    import { ethers } from "ethers";
    import { onMount } from 'svelte';

    let provider;
    let signer;

    onMount(async () => {
        console.log("connecting to ethereum")
        provider = new ethers.providers.Web3Provider(window.ethereum, "any");
        provider.on("network", (newNetwork, oldNetwork) => {
        // When a Provider makes its initial connection, it emits a "network"
        // event with a null oldNetwork along with the newNetwork. So, if the
        // oldNetwork exists, it represents a changing network
        console.log(newNetwork)    
        console.log(oldNetwork)
        if (oldNetwork) {
                window.location.reload();
            }
        });

        /// Requests the end user to switch to sepolia.
        await window.ethereum.request({
          method: 'wallet_switchEthereumChain',
          params: [{ chainId: "0xaa36a7" }]
        });

        signer = provider.getSigner()
        console.log("fetching ethereum balance")
        let balance = await provider.getBalance(signer.getAddress());
        console.log(balance.toString())
    })
</script>

<div>

</div>