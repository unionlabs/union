import { atom } from "nanostores"
import { nanoquery } from "@nanostores/query"
import { useStore } from "@nanostores/preact"
import { signal, computed, effect } from "alien-signals"

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

export function Processable(props: ProcessableProps) {

  const { fn, key, warning } = props

  computed(async () => {
    console.info('d')
    const result = await fn()
  })
  // const [result, setResult] = useStore(nanoquery(localStorage, key))

  return <div></div>
}
