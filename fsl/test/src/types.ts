export interface TestSuite {
  name: string
  tests: TestCase[]
  beforeEach?: () => void | Promise<void>
  afterEach?: () => void | Promise<void>
  beforeAll?: () => void | Promise<void>
  afterAll?: () => void | Promise<void>
}

export interface TestCase {
  name: string
  fn: () => void | Promise<void>
  only?: boolean
  skip?: boolean
  timeout?: number
}

export interface Assertion {
  toBe(expected: unknown): void
  toEqual(expected: unknown): void
  toBeDefined(): void
  toBeNull(): void
  toBeTruthy(): void
  toBeFalsy(): void
  toHaveLength(length: number): void
  toContain(item: unknown): void
  toThrow(message?: string): void
  not: Assertion
}

export interface RenderResult {
  container: Element
  getByText(text: string): Element
  getByTestId(id: string): Element
  queryByText(text: string): Element | null
  click(element: Element): Promise<void>
  type(element: Element, text: string): Promise<void>
  unmount(): void
}

export interface MockServerFunction<T extends (...args: unknown[]) => Promise<unknown>> {
  mock: T
  calls: Parameters<T>[]
}
