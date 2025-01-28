#!/usr/bin/env node
if (!BigInt.prototype.toJSON) {
    Object.defineProperty(BigInt.prototype, "toJSON", {
        value: function () {
            return this.toString();
        },
        writable: true,
        configurable: true
    });
}
import { request, gql } from "graphql-request";
import fetch, { Headers } from "node-fetch";
import fs from "fs";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import consola from "consola";
// For the EVM cross-chain transfer snippet:
import { fallback, http, fromHex } from "viem";
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing";
import { privateKeyToAccount } from "viem/accounts";
// If you’re pulling createUnionClient from your local or a published package:
import { createUnionClient, hexToBytes, getRecommendedChannels, getChannelInfo, getQuoteToken } from "@unionlabs/client";
// Hasura endpoint
const HASURA_ENDPOINT = "https://hubble-purple.hasura.app/v1/graphql";
// Set to track reported block hashes
const reportedsendTxHashes = new Set();
// Variable to track sleep cycles
let sleepCycleCount = 0;
// Set global fetch and Headers
if (!globalThis.fetch) {
    globalThis.fetch = fetch;
}
if (!globalThis.Headers) {
    globalThis.Headers = Headers;
}
function getRandomArbitrary(min_bigint, max_bigint) {
    const min = Number(min_bigint);
    const max = Number(max_bigint);
    const value = Math.random() * (max - min) + min;
    return BigInt(Math.ceil(value));
}
// Parse command-line arguments
function loadConfig(configPath) {
    if (!fs.existsSync(configPath)) {
        throw new Error("Config file not found. Ensure config.json exists.");
    }
    const rawData = fs.readFileSync(configPath, "utf-8");
    const config = JSON.parse(rawData);
    if (!Array.isArray(config.interactions) || config.interactions.length === 0) {
        throw new Error("Config file is invalid or interactions array is empty.");
    }
    return config;
}
/**
 * Check IBC packets between source_chain <-> destination_chain.
 *
 * We fetch the last 200× 'timeframe' from Hasura.
 * For each packet older than timeframe:
 *   - Check RECV / WRITE_ACK / ACK existence & timings.
 *   - Log any that are missing or exceed the time window.
 *
 * @param sourceChain A string ID (e.g. "11155111")
 * @param destinationChain Another string ID (e.g. "17000")
 * @param timeframeMs The SLA timeframe in milliseconds
 */
export async function checkPackets(sourceChain, destinationChain, timeframeMs) {
    // Current time
    const now = Date.now();
    // We'll query more than the timeframe to ensure we catch all
    const searchRangeMs = timeframeMs * 10;
    const sinceDate = new Date(now - searchRangeMs).toISOString();
    consola.info(`Querying Hasura for packets >= ${sinceDate}, chain-pair: ${sourceChain} <-> ${destinationChain}`);
    // Build the GraphQL query:
    const query = gql `
    query ($since: timestamptz!, $srcChain: String!, $dstChain: String!) {
      v1_ibc_union_packets(
        where: {
          _and: [
            {
              _or: [
                {
                  source_chain_id: { _eq: $srcChain }
                  destination_chain_id: { _eq: $dstChain }
                }
                {
                  source_chain_id: { _eq: $dstChain }
                  destination_chain_id: { _eq: $srcChain }
                }
              ]
            }
            { packet_send_timestamp: { _gte: $since } }
          ]
        }
        order_by: { packet_send_timestamp: asc }
      ) {
        packet_send_timestamp
        packet_recv_timestamp
        write_ack_timestamp
        packet_ack_timestamp
        source_chain_id
        destination_chain_id
        packet_send_transaction_hash
        packet_recv_transaction_hash
        write_ack_transaction_hash
        packet_ack_transaction_hash
      }
    }
  `;
    const variables = {
        since: sinceDate,
        srcChain: sourceChain,
        dstChain: destinationChain
    };
    //EEE48878CB7D9CE8DF02B87763FE6A8D8ECA7ACE77F9F483142415B0FFFD52FA
    try {
        // Post to Hasura
        const response = await request(HASURA_ENDPOINT, query, variables);
        const data = response.v1_ibc_union_packets ?? [];
        consola.info(`Found ${data.length} packets in the last ${searchRangeMs}ms for ${sourceChain} <-> ${destinationChain}`);
        // Check each packet
        for (const p of data) {
            const sendStr = p.packet_send_timestamp;
            if (!sendStr) {
                continue;
            }
            // Convert sendStr to a Date
            const sendTimeMs = new Date(sendStr).getTime();
            // Only check those older than or equal to SLA timeframe
            if (now - sendTimeMs < timeframeMs) {
                // Not old enough to be considered overdue
                continue;
            }
            // If we're here, the packet is older than `timeframeMs`.
            const recvStr = p.packet_recv_timestamp;
            const writeAckStr = p.write_ack_timestamp;
            const ackStr = p.packet_ack_timestamp;
            const sendTxHash = p.packet_send_transaction_hash ?? "?";
            if (reportedsendTxHashes.has(sendTxHash)) {
                continue;
            }
            // 1) RECV
            if (!recvStr) {
                consola.error(`[TRANSFER_ERROR: RECV MISSING] >${timeframeMs}ms since send. sendTxHash=${sendTxHash}, source_chain=${p.source_chain_id}, dest_chain=${p.destination_chain_id}`);
                reportedsendTxHashes.add(sendTxHash);
                continue;
            }
            else {
                const recvTimeMs = new Date(recvStr).getTime();
                if (recvTimeMs - sendTimeMs > timeframeMs) {
                    consola.error(`[RECV TOO LATE] >${timeframeMs}ms. send_time=${sendStr}, recv_time=${recvStr}, sendTxHash=${sendTxHash}`);
                    reportedsendTxHashes.add(sendTxHash);
                }
            }
            // 2) WRITE_ACK
            if (!writeAckStr) {
                consola.error(`[TRANSFER_ERROR: WRITE_ACK MISSING] >${timeframeMs}ms since send. sendTxHash=${sendTxHash}, source_chain=${p.source_chain_id}, dest_chain=${p.destination_chain_id}`);
                reportedsendTxHashes.add(sendTxHash);
                continue;
            }
            else {
                const writeAckTimeMs = new Date(writeAckStr).getTime();
                if (writeAckTimeMs - sendTimeMs > timeframeMs) {
                    consola.error(`[TRANSFER_ERROR: WRITE_ACK TOO LATE] >${timeframeMs}ms. sendTxHash=${sendTxHash}, send_time=${sendStr}, write_ack_time=${writeAckStr}`);
                    reportedsendTxHashes.add(sendTxHash);
                }
            }
            // 3) ACK
            if (!ackStr) {
                consola.error(`[TRANSFER_ERROR: ACK MISSING] >${timeframeMs}ms since send. sendTxHash=${sendTxHash}, source_chain=${p.source_chain_id}, dest_chain=${p.destination_chain_id}`);
                reportedsendTxHashes.add(sendTxHash);
            }
            else {
                const ackTimeMs = new Date(ackStr).getTime();
                if (ackTimeMs - sendTimeMs > timeframeMs) {
                    consola.error(`[TRANSFER_ERROR: ACK TOO LATE] >${timeframeMs}ms. send_time=${sendStr}, ack_time=${ackStr}, sendTxHash=${sendTxHash}`);
                    reportedsendTxHashes.add(sendTxHash);
                }
            }
        }
    }
    catch (error) {
        consola.error("Error fetching data from Hasura:", error.message);
    }
}
/**
 * Perform an EVM cross-chain transfer or estimate the gas for it.
 * Adapt the logic as needed to match your chain IDs / workflow.
 */
async function doTransfer(task) {
    if (!task.enabled) {
        consola.info("Transfer task is disabled. Skipping.");
        return;
    }
    const isCosmosChain = Boolean(task.sourceChainIdCosmos);
    const chainType = isCosmosChain ? "Cosmos" : "EVM";
    const random_amount = getRandomArbitrary(task.amount_range[0], task.amount_range[1]);
    try {
        consola.info("\n[%s] Starting transfer for chainId=%s to chain=%s", chainType, isCosmosChain ? task.sourceChainIdCosmos : task.sourceChainIdEVM, task.destinationChainId);
        const evmAccount = privateKeyToAccount(`0x${task.privateKey.replace(/^0x/, "")}`);
        const cosmosAccount = await DirectSecp256k1Wallet.fromKey(Uint8Array.from(hexToBytes(task.privateKey)), task.cosmosAccountType);
        const transports = task.rpcs.map(rpc => http(rpc));
        const sourceChainId = isCosmosChain ? task.sourceChainIdCosmos : task.sourceChainIdEVM;
        const channels = await getRecommendedChannels();
        const channel = getChannelInfo(sourceChainId, task.destinationChainId, channels);
        if (channel === null) {
            consola.error("No channel found. Source chain ID:", sourceChainId, " Destination chain ID:", task.destinationChainId);
            return;
        }
        const quoteToken = await getQuoteToken(sourceChainId, task.denomAddress.toLowerCase(), channel);
        consola.info("quoteToken: ", quoteToken, " chainId: ", sourceChainId, " denomAddr: ", task.denomAddress, " channel: ", channel);
        if (quoteToken.isErr()) {
            consola.info("could not get quote token");
            consola.error(quoteToken.error);
            return;
        }
        if (quoteToken.value.type === "NO_QUOTE_AVAILABLE") {
            consola.info("No quote token available");
            return;
        }
        consola.info("quote token", quoteToken.value);
        const txPayload = isCosmosChain
            ? {
                baseToken: task.denomAddress,
                baseAmount: BigInt(random_amount),
                quoteToken: quoteToken.value.quote_token,
                quoteAmount: BigInt(random_amount),
                receiver: task.receiverAddress,
                sourceChannelId: channel.source_channel_id,
                ucs03address: fromHex(`0x${channel.source_port_id}`, "string")
            }
            : {
                baseToken: task.denomAddress,
                baseAmount: BigInt(random_amount),
                quoteToken: quoteToken.value.quote_token,
                quoteAmount: BigInt(random_amount),
                receiver: task.receiverAddress,
                sourceChannelId: channel.source_channel_id,
                ucs03address: `0x${channel.source_port_id}`
            };
        let unionClient = null;
        if (isCosmosChain) {
            unionClient = createUnionClient({
                account: cosmosAccount,
                chainId: task.sourceChainIdCosmos,
                gasPrice: { amount: "0.025", denom: task.gasPriceDenom },
                transport: transports[0]
            });
        }
        else {
            unionClient = createUnionClient({
                account: evmAccount,
                chainId: task.sourceChainIdEVM,
                transport: fallback(transports)
            });
            const approveResponse = await unionClient.approveErc20(txPayload);
            consola.info("approve response: ", approveResponse);
            if (approveResponse.isErr()) {
                consola.error(approveResponse.error);
                return;
            }
        }
        const transferResp = await unionClient.transferAsset(txPayload);
        if (transferResp.isErr()) {
            consola.error("[%s] Transfer error:", chainType, transferResp.error);
            return;
        }
        consola.info("[%s] Transfer success:", chainType, transferResp.value);
    }
    catch (error) {
        const msg = error instanceof Error ? error.message : String(error);
        consola.error("[%s] Transfer exception: %s", chainType, msg);
    }
}
/**
 * This loop runs your IBC checks on the interval specified by `config.cycleIntervalMs`.
 * (For example, once every hour if config.cycleIntervalMs = 3600000)
 */
async function runIbcChecksForever(config) {
    const chainPairs = config.interactions;
    while (true) {
        consola.info("\n========== Starting IBC cross-chain checks ==========");
        for (const pair of chainPairs) {
            if (!pair.enabled) {
                consola.info("Checking task is disabled. Skipping.");
                continue;
            }
            consola.info(`Checking pair ${pair.sourceChain} <-> ${pair.destinationChain} with timeframe ${pair.timeframeMs}ms`);
            try {
                await checkPackets(pair.sourceChain, pair.destinationChain, pair.timeframeMs);
                consola.info(`Check complete for pair ${pair.sourceChain} <-> ${pair.destinationChain}`);
            }
            catch (err) {
                consola.error(`Error while checking pair ${pair.sourceChain} <-> ${pair.destinationChain}:`, err);
            }
        }
        // Optionally clear the reportedsendTxHashes set every 3 cycles
        sleepCycleCount++;
        if (sleepCycleCount % 3 === 0) {
            reportedsendTxHashes.clear();
            consola.info("Cleared reported block hashes.");
        }
        // Sleep for whatever cycleIntervalMs is set to (e.g. 1 hour)
        consola.info(`IBC checks done. Sleeping for ${config.cycleIntervalMs / 1000 / 60} minutes...`);
        await new Promise(resolve => setTimeout(resolve, config.cycleIntervalMs));
    }
}
/**
 * This loop runs your transfer tasks every 10 minutes,
 * regardless of how often IBC checks happen.
 */
async function runTransfersForever(config) {
    const transfers = config.transfers ?? [];
    const TEN_MINUTES_MS = 10 * 60 * 1000;
    while (true) {
        if (transfers.length > 0) {
            consola.info("\n========== Starting transfers tasks ==========");
            for (const task of transfers) {
                await doTransfer(task);
            }
        }
        else {
            consola.info("No transfers configured. Skipping transfer step.");
        }
        consola.info(`Transfers done (or skipped). Sleeping 10 minutes...`);
        await new Promise(resolve => setTimeout(resolve, TEN_MINUTES_MS));
    }
}
function sleepSync(ms) {
    const end = Date.now() + ms;
    while (Date.now() < end) {
        // Busy-wait for the specified duration
    }
}
/**
 * A "fire-and-forget" style load test function.
 *
 * This will trigger N parallel transfers *without* awaiting their completion.
 *
 * @param task The transfer configuration
 * @param totalRequests How many transfer calls to spawn
 * @param privKeys Optional array of private keys to rotate through
 */
function doTransferLoadTest(task, totalRequests, privKeys) {
    if (!task.enabled) {
        consola.info("Transfer task is disabled. Skipping.");
        return;
    }
    const useKeys = privKeys?.length ? privKeys : [task.privateKey];
    // Kick off multiple transfers in parallel
    for (let i = 0; i < totalRequests; i++) {
        const index = i % useKeys.length;
        const newPrivateKey = useKeys[index];
        const loadTask = { ...task, privateKey: newPrivateKey }; // overwrite the key
        consola.info("Starting transfer", i + 1, "with key", newPrivateKey);
        if (i > 0 && i % 10 === 0) {
            consola.info(`Sleeping for 10 seconds after ${i} transfers...`);
            sleepSync(5000); // Block the loop for 10 seconds
        }
        // Fire the asynchronous function but do NOT await
        doTransfer(loadTask).catch(err => {
            // Optionally catch errors so they don't become unhandled rejections
            consola.error(`[LoadTest] Transfer ${i + 1}/${totalRequests} failed:`, err);
        });
    }
    // Since we are not awaiting, this function will return immediately.
    consola.info(`Kicked off ${totalRequests} parallel transfers for load test.`);
}
/**
 * Kick off both loops in parallel.
 */
async function main() {
    const argv = await yargs(hideBin(process.argv))
        .option("config", {
        alias: "c",
        type: "string",
        demandOption: true,
        describe: "Path to the configuration file"
    })
        .help()
        .alias("help", "h")
        .parse();
    const configPath = argv.config;
    consola.info(`Using config file: ${configPath}`);
    // Load configuration
    const config = loadConfig(configPath);
    const is_loadtest = config.load_test_request > 0 ? true : false;
    if (is_loadtest) {
        // Run a one-time load test
        const transfers = config.transfers ?? [];
        if (transfers.length === 0) {
            consola.warn("No transfers configured. Nothing to load-test.");
            return;
        }
        consola.info("========== Starting Load Test ==========");
        for (const task of transfers) {
            doTransferLoadTest(task, config.load_test_request, config.privkeys_for_loadtest);
        }
        // You can exit after scheduling them if you don't want
        // to remain running. Or keep the process alive if needed.
        // If you prefer to exit:
        // process.exit(0)
    }
    else {
        // Normal mode: run IBC checks + transfer tasks in parallel
        await Promise.all([runIbcChecksForever(config), runTransfersForever(config)]);
    }
}
// Just call `main()` immediately
main().catch(err => consola.error("Error in main()", err));
