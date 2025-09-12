import * as V from "vitest"
import "@testing-library/jest-dom/vitest"
import * as domMatchers from "@testing-library/jest-dom/matchers"
V.expect.extend(domMatchers)
