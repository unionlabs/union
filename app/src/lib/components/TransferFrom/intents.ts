import { writable } from "svelte/store"
import { browser } from "$app/environment"
import { page } from "$app/stores"
import { transferSchema } from "./validation.ts"
import { safeParse } from "valibot"

//Need to clean up the types so they make sense
//RawTransferIntents should not contain errors etc just the raw inputs

type FormFields = {
  source: string
  destination: string
  asset: string
  receiver: string
  amount: string
}

type FieldErrors = Partial<Record<keyof FormFields, string>>

interface RawTransferIntents extends FormFields {
  errors: FieldErrors
  isValid: boolean
}

export interface IntentStore {
  subscribe: (callback: (value: RawTransferIntents) => void) => () => void
  set: (value: Partial<FormFields>) => void
  updateField: (key: keyof FormFields, value: string) => void
  reset: () => void
  validate: () => Promise<boolean>
}

const defaultParams: RawTransferIntents = {
  source: "",
  destination: "",
  asset: "",
  receiver: "",
  amount: "",
  errors: {},
  isValid: false
}

export function createIntentStore(): IntentStore {
  const store = writable<RawTransferIntents>(defaultParams)
  const { subscribe, set, update } = store

  function updateUrl({ source, destination, asset, receiver, amount }: FormFields) {
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
    }
  }

  function validate(params: FormFields): FieldErrors {
    const result = safeParse(transferSchema, params)

    if (!result.success) {
      return result.issues.reduce((acc, issue) => {
        const fieldName = issue.path?.[0]?.key as keyof FormFields

        if (fieldName && !params[fieldName]) {
          return acc
        }

        if (fieldName) {
          acc[fieldName] = issue.message
        }
        return acc
      }, {} as FieldErrors)
    }

    return {}
  }

  if (browser) {
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

        update(state => {
          const validatedErrors = validate({ ...state, ...newParams })
          return {
            ...state,
            ...newParams,
            errors: validatedErrors,
            isValid: Object.keys(validatedErrors).length === 0
          }
        })
      }
    })
  }

  return {
    subscribe,

    set: (value: Partial<FormFields>) => {
      update(state => {
        const newParams = { ...state, ...value }
        const errors = validate(newParams)
        updateUrl(newParams)
        return {
          ...newParams,
          errors,
          isValid: Object.keys(errors).length === 0
        }
      })
    },

    updateField: (key: keyof FormFields, value: string) => {
      update(state => {
        const newParams = { ...state, [key]: value }
        const errors = validate(newParams)
        updateUrl(newParams)
        return {
          ...newParams,
          errors,
          isValid: Object.keys(errors).length === 0
        }
      })
    },

    reset: () => {
      update(_state => {
        const errors = validate({ ...defaultParams })
        const isValid = Object.keys(errors).length === 0
        if (browser) {
          history.replaceState({}, "", window.location.pathname)
        }
        return {
          ...defaultParams,
          errors,
          isValid
        }
      })
    },

    validate: () => {
      return new Promise(resolve => {
        update(state => {
          const { source, destination, asset, receiver, amount } = state
          const errors = validate({ source, destination, asset, receiver, amount })
          const isValid = Object.keys(errors).length === 0
          resolve(isValid)
          return { ...state, errors, isValid }
        })
      })
    }
  }
}
