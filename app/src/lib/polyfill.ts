import "temporal-polyfill/global"
import EventEmitter from "events"
import { browser } from "$app/environment"

if (browser) {
  window.EventEmitter = EventEmitter
}

// @ts-expect-error
BigInt["prototype"].toJSON = function () {
  return this.toString()
}
