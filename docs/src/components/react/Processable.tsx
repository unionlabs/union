import * as React from "react"
import { atom } from "nanostores"
import { useStore } from "@nanostores/react"
import { nanoquery } from "@nanostores/query"
import { signal, computed, effect } from "alien-signals"

// export const prer

/**
 * this component takes data of type ProcessableProps,
 * accepts a(n async) function as a prop,
 * a localStorage key as a prop to store the result of the function,
 *
 * and a warning the localStorage key is not found.
 */

// export const localStorageKey = atom<string>('yo')
interface ProcessableProps {
  fn: () => Promise<string | void>
  key: string
  warning?: string
}

export function Processable({
  props,
  children
}: { children: React.ReactNode; props: ProcessableProps }) {
  React.useEffect(() => {
    console.info('ssss')
    const [result, setResult] = useStore(nanoquery(localStorage, props.key))

    if (result) {
      setResult(undefined)
    }

    const fn = async () => {
      const result = await props.fn()
      console.info(result)
      if (result) {
        setResult(result)
      }
    }
    fn()
  }, [props.key, props.fn])

  return <div></div>
}
