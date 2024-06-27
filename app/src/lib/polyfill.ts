import "viem/window"
import process from "node:process"
import { Buffer } from "node:buffer"
import EventEmitter from "node:events"
import { browser } from "$app/environment"

if (browser) {
  window.Buffer = Buffer
  window.global = window
  window.process = process
  window.EventEmitter = EventEmitter
}

// @ts-expect-error
BigInt["prototype"].toJSON = function () {
  return this.toString()
}
