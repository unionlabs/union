import { describe, expect, test } from "vitest"
import "@testing-library/jest-dom/vitest"
import { render, screen } from "@testing-library/svelte"
import Page from "./+page.svelte"

describe("/+page.svelte", () => {
  test("should render h1", () => {
    render(Page)
    expect(screen.getByRole("heading", { level: 1 })).toBeInTheDocument()
  })
})
