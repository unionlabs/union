import { cleanup, render, screen } from "@testing-library/svelte/svelte5"
import { Option } from "effect"
import { describe, expect, test } from "vitest"

import MapOption from "./snippets/MapOption.svelte"
import MatchOption from "./snippets/MatchOption.svelte"

describe("Snippets", () => {
  describe.sequential("Option", () => {
    test.sequential.each(
      [
        ["Some", Option.some("a")],
        ["None", Option.none()],
      ] as const,
    )("matchOption handles $0", (s, value) => {
      render(MatchOption, { props: { value } })
      expect(screen.getByTestId(s)).toBeInTheDocument()
      cleanup()
    })

    test.sequential.each(
      [
        ["Some", Option.some("a")],
        ["None", Option.none()],
      ] as const,
    )("mapOption handles $0", (s, value) => {
      render(MapOption, { props: { value } })
      Option.match(value, {
        onSome(a) {
          expect(screen.getByTestId(s)).toBeInTheDocument()
          expect(screen.getByText(a)).toBeInTheDocument()
        },
        onNone() {
          expect(screen.queryByTestId(s)).not.toBeInTheDocument()
        },
      })
      cleanup()
    })
  })
})
