import { writable } from "svelte/store"
import { browser } from "$app/environment"
import { page } from "$app/stores"
import { debounce } from "$lib/utilities"
import { defaultParams } from "$lib/components/TransferFrom/transfer/config.ts"

export type FormFields = {
  source: string
  destination: string
  asset: string
  receiver: string
  amount: string
}

export interface RawIntentsStore {
  subscribe: (callback: (value: FormFields) => void) => () => void
  set: (value: Partial<FormFields>) => void
  updateField: (field: keyof FormFields, valueOrEvent: string | Event | null) => void
  reset: () => void
}

// Helper function to clean the state object
const cleanState = (state: Partial<FormFields>): FormFields => {
  const { source = "", destination = "", asset = "", receiver = "", amount = "" } = state
  return { source, destination, asset, receiver, amount }
}

export function createRawIntentsStore(): RawIntentsStore {
  const store = writable<FormFields>(cleanState(defaultParams))
  const { subscribe, update } = store
  let isUpdatingFromURL = false

  const debouncedUpdateUrl = debounce((params: FormFields) => {
    if (browser && !isUpdatingFromURL) {
      const url = new URL(window.location.href)
      Object.entries(params).forEach(([key, val]) => {
        if (val) {
          url.searchParams.set(key, val)
        } else {
          url.searchParams.delete(key)
        }
      })
      history.replaceState({}, "", url.toString())
    }
  }, 1000)

  if (browser) {
    // Initial setup remains the same
    const setDefaultParamsIfEmpty = (searchParams: URLSearchParams) => {
      if ([...searchParams.entries()].length === 0) {
        const url = new URL(window.location.href)
        const cleanedParams = cleanState(defaultParams)
        Object.entries(cleanedParams).forEach(([key, val]) => {
          if (val) {
            url.searchParams.set(key, val)
          }
        })
        history.replaceState({}, "", url.toString())
      }
    }

    setDefaultParamsIfEmpty(new URL(window.location.href).searchParams)

    // URL sync handling
    page.subscribe(pageData => {
      if (pageData?.url?.searchParams && !isUpdatingFromURL) {
        isUpdatingFromURL = true
        const newParams: Partial<FormFields> = {}
        const queryParams = pageData.url.searchParams
        ;(Object.keys(defaultParams) as Array<keyof FormFields>).forEach(key => {
          const value = queryParams.get(key)
          if (value) {
            newParams[key] = value
          }
        })

        store.set(cleanState({ ...defaultParams, ...newParams }))
        isUpdatingFromURL = false
      }
    })
  }

  return {
    subscribe,

    set: (value: Partial<FormFields>) => {
      let newParams: FormFields = cleanState({})

      // Immediate state update
      update(state => {
        newParams = cleanState({ ...state, ...value })
        return newParams
      })

      // Debounced URL update
      debouncedUpdateUrl(newParams)
    },

    updateField: (field: keyof FormFields, valueOrEvent: string | Event | null) => {
      if (!field) return
      const value =
        valueOrEvent instanceof Event
          ? (valueOrEvent.target as HTMLInputElement).value
          : valueOrEvent

      let newParams: FormFields = cleanState({})

      // Immediate state update
      update(state => {
        newParams = cleanState({ ...state, [field]: value })

        const resetMapping: Partial<Record<keyof FormFields, Array<keyof FormFields>>> = {
          source: ["asset", "amount"],
          asset: ["amount"],
          destination: ["receiver"]
        } as const

        const fieldsToReset = resetMapping[field]
        if (fieldsToReset) {
          fieldsToReset.forEach(resetField => {
            newParams = { ...newParams, [resetField]: "" }
          })
        }

        return newParams
      })

      // Debounced URL update
      debouncedUpdateUrl(newParams)
    },

    reset: () => {
      const cleanedParams = cleanState(defaultParams)
      store.set(cleanedParams)

      if (browser) {
        const url = new URL(window.location.href)
        url.search = ""
        Object.entries(cleanedParams).forEach(([key, val]) => {
          if (val) {
            url.searchParams.set(key, val)
          }
        })
        history.replaceState({}, "", url.toString())
      }
    }
  }
}
