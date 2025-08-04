import { Client } from "@unionlabs/sdk"
import { Layer } from "effect"
import * as internal from "./internal/client.js"

/**
 * @since 1.0.0
 * @category layers
 */
export const layer: Layer.Layer<Client.Client> = internal.layer
