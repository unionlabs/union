import { writable } from "svelte/store"
import { browser } from "$app/environment"
import { page } from "$app/stores"
import { debounce } from "$lib/utilities"
import { defaultParams } from "$lib/components/TransferFrom/config.ts"

export type FormFields = {
  source: string
  destination: string
  asset: string
  receiver: string
  amount: string
}

export interface RawTransferIntents extends FormFields {
  isValid: boolean
}

export interface IntentStore {
  subscribe: (callback: (value: RawTransferIntents) => void) => () => void
  set: (value: Partial<FormFields>) => void
  updateField: (field: keyof FormFields, valueOrEvent: string | Event) => void
  reset: () => void
}

export function createIntentStore(): IntentStore {
  const store = writable<RawTransferIntents>(defaultParams)
  const { subscribe, set, update } = store

  const debouncedUpdateUrl = debounce(
    ({ source, destination, asset, receiver, amount }: FormFields) => {
      if (browser) {
        const url = new URL(window.location.href)
        const params = { source, destination, asset, receiver, amount }

        Object.entries(params).forEach(([key, val]) => {
          if (val) {
            url.searchParams.set(key, val)
          } else {
            url.searchParams.delete(key)
          }
        })
        history.replaceState({}, "", url.toString())
        window.dispatchEvent(new PopStateEvent("popstate"))
      }
    },
    1000
  )

  if (browser) {
    const setDefaultParamsIfEmpty = (searchParams: URLSearchParams) => {
      if ([...searchParams.entries()].length === 0) {
        const url = new URL(window.location.href)
        Object.entries(defaultParams).forEach(([key, val]) => {
          if (val) {
            url.searchParams.set(key, val)
          }
        })
        history.replaceState({}, "", url.toString())
        window.dispatchEvent(new PopStateEvent("popstate"))
      }
    }

    setDefaultParamsIfEmpty(new URL(window.location.href).searchParams)

    page.subscribe(pageData => {
      if (pageData?.url?.searchParams) {
        const newParams: Partial<FormFields> = {}
        const queryParams = pageData.url.searchParams
        ;(Object.keys(defaultParams) as Array<keyof FormFields>).forEach(key => {
          const value = queryParams.get(key)
          if (value) {
            newParams[key] = value
          }
        })

        update(state => ({
          ...state,
          ...newParams
        }))
      }
    })
  }

  return {
    subscribe,

    set: (value: Partial<FormFields>) => {
      update(state => {
        const newParams = { ...state, ...value }
        debouncedUpdateUrl(newParams)
        return newParams
      })
    },

    updateField: (field: keyof FormFields, valueOrEvent: string | Event) => {
      const value =
        valueOrEvent instanceof Event
          ? (valueOrEvent.target as HTMLInputElement).value
          : valueOrEvent

      update(state => {
        const newParams = { ...state, [field]: value }
        debouncedUpdateUrl(newParams)
        return newParams
      })
    },

    reset: () => {
      if (browser) {
        const url = new URL(window.location.href)
        url.search = ""
        Object.entries(defaultParams).forEach(([key, val]) => {
          if (val) {
            url.searchParams.set(key, val)
          }
        })
        history.replaceState({}, "", url.toString())
        window.dispatchEvent(new PopStateEvent("popstate"))
      }
      set(defaultParams)
    }
  }
}