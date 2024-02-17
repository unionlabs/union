import process from 'process'
import { Buffer } from 'buffer'
import EventEmitter from 'events'
import { browser } from '$app/environment'

if (browser) {
  window.Buffer = Buffer
  window.process = process
  window.EventEmitter = EventEmitter
  window.global = window
}
