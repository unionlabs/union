import { StargateClient } from "@cosmjs/stargate";
import { Tendermint37Client } from "@cosmjs/tendermint-rpc";

const client = await Tendermint37Client.connect(
  "https://union-testnet-rpc.polkachu.com"
);
// const client = await StargateClient.connect('https://union-testnet-rpc.polkachu.com')

client.subscribeNewBlock().subscribe({
  next: (block) => {
    console.log("New block:", block);
  },
  error: (error) => {
    console.error("Error:", error);
  },
  complete: () => {
    console.log("Complete");
  },
});
