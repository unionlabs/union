import {
  erc20Abi,
  type Hash,
  bytesToHex,
  type Account,
  type Address,
  type TransportConfig,
} from "viem";
import { raise } from "./utilities";
import { ucs01relayAbi } from "./abi";
import { GasPrice } from "@cosmjs/stargate";
import { fromBech32 } from "@cosmjs/encoding";
import type { UnionClient } from "./actions.ts";
import { Tendermint37Client } from "@cosmjs/tendermint-rpc";
import { UNION_RPC_URL, UCS01_EVM_ADDRESS } from "./constants";
import { chainIds, type ChainId, chain } from "./constants/chain.ts";
import type { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import type { CosmjsOfflineSigner } from "@leapwallet/cosmos-snap-provider";
import {
  type ExecuteResult,
  SigningCosmWasmClient,
} from "@cosmjs/cosmwasm-stargate";

export interface ApproveAssetParameters {
  chainId: "1" | "11155111";
  assetId: Address;
  signer: Account | Address;
  amount: bigint;
  spender?: Address;
  simulate?: boolean;
}

export async function approveAsset(
  client: UnionClient,
  {
    signer,
    assetId,
    amount,
    spender = UCS01_EVM_ADDRESS,
    simulate = true,
  }: ApproveAssetParameters
): Promise<Hash> {
  try {
    const writeContractParameters = {
      account: signer,
      abi: erc20Abi,
      functionName: "approve",
      address: assetId,
      args: [spender, amount],
      chain: client.chain,
    } as const;

    if (!simulate) return await client.writeContract(writeContractParameters);

    const { request } = await client.simulateContract(writeContractParameters);
    const transactionHash = await client.writeContract(request);
    return transactionHash;
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : error;
    raise(
      `[approveAsset] error while approving ${amount} muno to ${spender}: ${errorMessage}`
    );
  }
}

export type SendAssetParameters<
  TChainId extends ChainId,
  TDenom extends string | undefined = TChainId extends "6" ? string : undefined,
  TGas extends `${string}${TDenom}` | undefined = TChainId extends "6"
    ? `${string}${TDenom}`
    : undefined,
  TTransportConfigType extends
    | TransportConfig["type"]
    | undefined = TChainId extends "6" ? TransportConfig["type"] : undefined
> =
  | ({ chainId: "6" } & SendAssetFromUnionToEthereum<TDenom, TGas>)
  | ({ chainId: "1" | "11155111" } & SendAssetFromEthereumToUnion);

export async function sendAsset<
  TChainId extends ChainId,
  TDenom extends string | undefined,
  TGas extends `${string}${TDenom}` | undefined,
  TTransportConfigType extends TransportConfig["type"] | undefined
>(
  client: UnionClient,
  args: SendAssetParameters<TChainId, TDenom, TGas, TTransportConfigType>
) {
  if (!chainIds.includes(args.chainId))
    throw new Error(`Invalid chainId: ${args.chainId}`);
  else if (args.chainId === "6")
    return await sendAssetFromUnionToEthereum(client, args);
  else if (args.chainId === "11155111")
    return await sendAssetFromEthereumToUnion(client, args);
  else raise(`[sendAsset] chainId ${args.chainId} is not supported`);
}

interface SendAssetFromEthereumToUnion {
  denomAddress: Address;
  receiver: string;
  amount: bigint;
  signer: Account | Address;
  portId: string;
  channelId: string;
  simulate?: boolean;
}

/**
 * Contract arguments:
 * @reference https://github.com/unionlabs/union/blob/main/evm/contracts/apps/ucs/01-relay/Relay.sol#L439-L444
 * - 'string calldata sourcePort',
 * - 'string calldata sourceChannel',
 * - `string calldata receiver`,
 * - `LocalToken[] calldata tokens`,
 * - `uint64 counterpartyTimeoutRevisionNumber`,
 * - `uint64 counterpartyTimeoutRevisionHeight`
 */
export async function sendAssetFromEthereumToUnion(
  client: UnionClient,
  {
    receiver,
    signer,
    amount,
    denomAddress,
    portId,
    channelId,
    simulate = true,
  }: SendAssetFromEthereumToUnion
): Promise<Hash> {
  // TODO: make dynamic?
  const counterpartyTimeoutRevisionNumber = BigInt(chain.union.testnet.id);
  // TODO: make dynamic?
  const counterpartyTimeoutRevisionHeight = 800_000_000n;
  try {
    const writeContractParameters = {
      account: signer,
      abi: ucs01relayAbi,
      functionName: "send",
      address: UCS01_EVM_ADDRESS,
      args: [
        portId,
        channelId,
        bytesToHex(fromBech32(receiver).data),
        [{ denom: denomAddress, amount }],
        counterpartyTimeoutRevisionNumber,
        counterpartyTimeoutRevisionHeight,
      ],
      chain: client.chain,
    } as const;

    if (!simulate) {
      return await client.writeContract(writeContractParameters);
    }
    const { request } = await client.simulateContract(writeContractParameters);
    const transactionHash = await client.writeContract(request);
    return transactionHash;
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : error;
    raise(
      `[sendAssetFromEthereumToUnion] error while sending ${amount} muno to ${receiver} on ${client.transport.name} request: ${errorMessage}`
    );
  }
}

type OfflineSignerType = CosmjsOfflineSigner | DirectSecp256k1HdWallet;

type SendAssetFromUnionToEthereum<
  TDenom extends string | undefined,
  TGas extends `${string}${TDenom}` | undefined
> = {
  contractAddress: string;
  receiver: string;
  amount: string;
  denom: `${TDenom}`;
  gasPrice?: TGas;
  rpcUrl: string;
  memo?: string;
  offlineSigner: OfflineSignerType;
  channel: string;
};

export async function sendAssetFromUnionToEthereum<
  TDenom extends string | undefined,
  TGas extends `${string}${TDenom}` | undefined
>(
  _client: UnionClient,
  {
    offlineSigner,
    contractAddress,
    amount,
    denom,
    receiver,
    gasPrice,
    channel,
    rpcUrl,
    memo,
  }: SendAssetFromUnionToEthereum<TDenom, TGas>
): Promise<ExecuteResult> {
  const tendermintClient = await Tendermint37Client.connect(rpcUrl);
  const cosmwasmClient = await SigningCosmWasmClient.createWithSigner(
    tendermintClient,
    offlineSigner,
    { gasPrice: GasPrice.fromString(gasPrice ?? `0.001${denom}`) }
  );

  const [account] = await offlineSigner.getAccounts();
  const address = account?.address ?? raise("address is undefined");

  const result = await cosmwasmClient.execute(
    address,
    contractAddress,
    { transfer: { channel, receiver: receiver.slice(2), timeout: null, memo } },
    "auto",
    undefined,
    [{ denom, amount }]
  );

  return result;
}
