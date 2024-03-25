/// <reference types="@sveltejs/kit" />
/// <reference no-default-lib="true"/>
/// <reference lib="webworker" />
/// <reference lib="esnext" />

import { build, files, version } from "$service-worker"

const sw = self as unknown as ServiceWorkerGlobalScope

console.info("Service Worker started")

const CACHE = `cache-${version}`

const ASSETS = [...build, ...files]

sw.addEventListener("install", _event => {
  console.info("Service Worker installed")
})

sw.addEventListener("activate", _event => {
  console.info("Service Worker activated")
})

sw.addEventListener("fetch", _event => {
  console.info("Service Worker fetching")
})
