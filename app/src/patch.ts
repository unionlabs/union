import process from 'node:process'
import { Buffer } from 'node:buffer'
import EventEmitter from 'node:events'
import { browser } from '$app/environment'

if (browser) {
  window.Buffer = Buffer
  window.process = process
  window.EventEmitter = EventEmitter
  window.global = window
}
