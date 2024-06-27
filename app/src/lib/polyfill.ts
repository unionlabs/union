import "viem/window"
/**
 * IMPORTANT: note about `process`, `buffer`, and `events` imports
 * these 3 imports must not use the Node.js import protocol (e.g., "node:process")
 */
import process from "process"
import { Buffer } from "buffer"
import EventEmitter from "events"
import { browser } from "$app/environment"

if (browser) {
  window.Buffer = Buffer
  window.global = window
  window.process = process
  window.EventEmitter = EventEmitter
}

BigInt["prototype"].toJSON = function () {
  return this.toString()
}
