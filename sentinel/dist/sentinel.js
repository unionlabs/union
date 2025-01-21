import { request, gql } from "graphql-request";
import fetch, { Headers } from "node-fetch";
import fs from "fs";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";
// Hasura endpoint
const HASURA_ENDPOINT = "https://hubble-purple.hasura.app/v1/graphql";
// Set to track reported block hashes
const reportedBlockHashes = new Set();
// Variable to track sleep cycles
let sleepCycleCount = 0;
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
// Set global fetch and Headers
if (!globalThis.fetch) {
    globalThis.fetch = fetch;
}
if (!globalThis.Headers) {
    globalThis.Headers = Headers;
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
    const searchRangeMs = timeframeMs * 500;
    const sinceDate = new Date(now - searchRangeMs).toISOString();
    console.info(`Querying Hasura for packets >= ${sinceDate}, chain-pair: ${sourceChain} <-> ${destinationChain}`);
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
                },
                {
                  source_chain_id: { _eq: $dstChain }
                  destination_chain_id: { _eq: $srcChain }
                }
              ]
            },
            { packet_send_timestamp: { _gte: $since } }
          ]
        }
        order_by: { packet_send_timestamp: asc }
        limit: 500
      ) {
        packet_send_timestamp
        packet_recv_timestamp
        write_ack_timestamp
        packet_ack_timestamp
        source_chain_id
        destination_chain_id
        packet_send_block_hash
        packet_recv_block_hash
      }
    }
  `;
    const variables = {
        since: sinceDate,
        srcChain: sourceChain,
        dstChain: destinationChain
    };
    try {
        // Post to Hasura
        const response = await request(HASURA_ENDPOINT, query, variables);
        const data = response.v1_ibc_union_packets ?? [];
        console.info(`Found ${data.length} packets in the last ${searchRangeMs}ms for ${sourceChain} <-> ${destinationChain}`);
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
            const blockHash = p.packet_send_block_hash ?? "?";
            if (reportedBlockHashes.has(blockHash)) {
                continue;
            }
            // 1) RECV
            if (!recvStr) {
                console.error(`[RECV MISSING] >${timeframeMs}ms since send. BlockHash=${p.packet_send_block_hash ?? "?"}, source_chain=${p.source_chain_id}, dest_chain=${p.destination_chain_id}`);
                // Add block hash to the set
                reportedBlockHashes.add(blockHash);
                continue;
            }
            else {
                const recvTimeMs = new Date(recvStr).getTime();
                if (recvTimeMs - sendTimeMs > timeframeMs) {
                    console.error(`[RECV TOO LATE] >${timeframeMs}ms. send_time=${sendStr}, recv_time=${recvStr}, blockHash=${p.packet_send_block_hash ?? "?"}`);
                    // Add block hash to the set
                    reportedBlockHashes.add(blockHash);
                }
            }
            // 2) WRITE_ACK
            if (!writeAckStr) {
                console.error(`[WRITE_ACK MISSING] >${timeframeMs}ms since send. BlockHash=${p.packet_send_block_hash ?? "?"}, source_chain=${p.source_chain_id}, dest_chain=${p.destination_chain_id}`);
                // Add block hash to the set
                reportedBlockHashes.add(blockHash);
                continue;
            }
            else {
                const writeAckTimeMs = new Date(writeAckStr).getTime();
                if (writeAckTimeMs - sendTimeMs > timeframeMs) {
                    console.error(`[WRITE_ACK TOO LATE] >${timeframeMs}ms. blockHash=${p.packet_send_block_hash ?? "?"}, send_time=${sendStr}, write_ack_time=${writeAckStr}`);
                    // Add block hash to the set
                    reportedBlockHashes.add(blockHash);
                }
            }
            // 3) ACK
            if (!ackStr) {
                console.error(`[ACK MISSING] >${timeframeMs}ms since send. BlockHash=${p.packet_send_block_hash ?? "?"}, source_chain=${p.source_chain_id}, dest_chain=${p.destination_chain_id}`);
                // Add block hash to the set
                reportedBlockHashes.add(blockHash);
            }
            else {
                const ackTimeMs = new Date(ackStr).getTime();
                if (ackTimeMs - sendTimeMs > timeframeMs) {
                    console.error(`[ACK TOO LATE] >${timeframeMs}ms. send_time=${sendStr}, ack_time=${ackStr}, blockHash=${p.packet_send_block_hash ?? "?"}`);
                    // Add block hash to the set
                    reportedBlockHashes.add(blockHash);
                }
                else {
                    console.debug(`Packet fully acked on time. blockHash=${p.packet_send_block_hash ?? "?"}`);
                }
            }
        }
    }
    catch (error) {
        console.error("Error fetching data from Hasura:", error.message);
    }
}
/**
 * Main function that calls `checkPackets` repeatedly,
 * similar to a cron job (infinite loop).
 */
export async function main() {
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
    console.info(`Using config file: ${configPath}`);
    // Load configuration
    const config = loadConfig(configPath);
    const chainPairs = config.interactions;
    const oneHourMs = config.cycleIntervalMs;
    while (true) {
        console.info("Starting IBC cross-chain checks...");
        for (const pair of chainPairs) {
            console.info(`Checking pair ${pair.sourceChain} <-> ${pair.destinationChain} with timeframe ${pair.timeframeMs}ms`);
            try {
                await checkPackets(pair.sourceChain, pair.destinationChain, pair.timeframeMs);
                console.info(`Check complete for pair ${pair.sourceChain} <-> ${pair.destinationChain}`);
            }
            catch (err) {
                console.error(`Error while checking pair ${pair.sourceChain} <-> ${pair.destinationChain}:`, err);
            }
        }
        sleepCycleCount++;
        if (sleepCycleCount % 3 === 0) {
            reportedBlockHashes.clear();
        }
        console.info("All checks done. Sleeping for 1 hour...");
        await new Promise(resolve => setTimeout(resolve, oneHourMs));
    }
}
// Just call `main()` immediately, since we don't have `require.main` in ESM
main().catch(err => console.error("Error in main()", err));
