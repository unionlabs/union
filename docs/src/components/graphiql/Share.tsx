import * as React from "react"

export function ShareButton({ encodedQuery }: { encodedQuery: string }) {
  return (
    <button
      type="button"
      className="top-0 right-0 z-10 rounded-md bg-transparent p-2 text-cyan-500 hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-cyan-500 disabled:pointer-events-none disabled:opacity-50 dark:focus:ring-cyan-600 dark:focus:ring-offset-gray-900"
    >
      SHARE
    </button>
  )
}
