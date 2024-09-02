import process from "process"
import { Buffer } from "buffer"
import EventEmitter from "events"

if (typeof window !== "undefined") {
  window.Buffer = Buffer
  window.global = window
  window.process = process
  window.EventEmitter = EventEmitter
}

BigInt["prototype"].toJSON = function () {
  return this.toString()
}
