export * from "./chain.ts";
import { raise } from "../utilities";

const CLIENT_MODE = process.env.CLIENT_MODE;

if (
  CLIENT_MODE === "browser" &&
  (typeof window === "undefined" || typeof window.ethereum === "undefined")
) {
  raise('window.ethereum is undefined and CLIENT_MODE is set to "browser"');
}

export const demoPrivateKey =
  "0x1075394148aee9ccae14500c37cfdfca7bea4a4984fd5882a9ecf1be610d84ee";
export const demoMnemonic =
  "enlist hip relief stomach skate base shallow young switch frequent cry park";
export const demoEthereumAddress = "0x3a7c1964ea700Ee19887c747C72e68F84Cb9C5DD";
export const demoUnionAddress = "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv";

/**
 * In case user elects to provide a custom RPC URL, they can set it via environment variable
 */
export const UNION_RPC_URL = process.env.UNION_RPC_URL;
export const UNION_REST_URL = process.env.UNION_REST_URL;
export const UNION_GRAPHQL_API = process.env.UNION_GRAPHQL_API;
export const UCS01_EVM_ADDRESS = process.env.UCS01_EVM_ADDRESS;
