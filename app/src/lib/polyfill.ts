import "viem/window"
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
