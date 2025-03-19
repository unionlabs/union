import { ucs03abi } from '@unionlabs/sdk/evm/abi';
import yargs from "yargs"
import { hideBin } from "yargs/helpers"
import consola from "consola"
import fs from "node:fs"
import { type Address, fallback, http, fromHex, toHex } from "viem"

console.log(ucs03abi);
// Shape of the chain pair config
interface ChainPair {
    sourceChain: string
    destinationChain: string
    timeframeMs: number
    enabled: boolean
  }
  
  // Shape of the EVM transfer config
  interface TransferConfig {
    enabled: boolean
    privateKey: string
    sourceChainIdEVM: string // TODO: change them later
    sourceChainIdCosmos: string // TODO: change them later
    destinationChainId: string // TODO: change them later
    rpcs: Array<string>
    gasPriceDenom: string
    receiverAddress: Address
    denomAddress: Address
    amount_range: Array<bigint>
    cosmosAccountType: string
  }
  
  // Combined config shape
  interface ConfigFile {
    interactions: Array<ChainPair>
    cycleIntervalMs: number
    transfers?: Array<TransferConfig> // optional array
    privkeys_for_loadtest?: Array<string>
    load_test_enabled: boolean
  }

// Parse command-line arguments
function loadConfig(configPath: string): ConfigFile {
    if (!fs.existsSync(configPath)) {
      throw new Error("Config file not found. Ensure config.json exists.")
    }
  
    const rawData = fs.readFileSync(configPath, "utf-8")
    const config = JSON.parse(rawData)
  
    if (!Array.isArray(config.interactions) || config.interactions.length === 0) {
      throw new Error("Config file is invalid or interactions array is empty.")
    }
  
    return config
  }
  

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
      .parse()
  
    const configPath = argv.config
    consola.info(`Using config file: ${configPath}`)
  
    // Load configuration
    const config = loadConfig(configPath)

    consola.info("config:", config)
    
}
// Just call `main()` immediately
main().catch(err => consola.error("Error in main()", err))