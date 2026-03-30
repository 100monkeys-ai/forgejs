// Forge-specific test utilities: component rendering, server function mocking,
// module mocking, and snapshot testing.

"use module test"

// RenderResult — returned by renderComponent()
export interface RenderResult {
  container: Element
  getByText(text: string): Element
  getByTestId(id: string): Element
  queryByText(text: string): Element | null
  click(element: Element): Promise<void>
  type(element: Element, text: string): Promise<void>
  unmount(): void
}

// renderComponent — render a .fx component into a detached DOM node.
// The returned RenderResult provides querying and interaction helpers.
// Signal state is fully functional; server functions are replaced by mocks.
export async function renderComponent(
  component: unknown,
  props?: Record<string, unknown>
): Promise<RenderResult> {
  // TODO: Mount the component using the Forge runtime in test mode
  const container = document.createElement('div')
  document.body.appendChild(container)

  return {
    container,
    getByText(text) {
      const el = Array.from(container.querySelectorAll('*'))
        .find(el => el.textContent?.trim() === text)
      if (!el) throw new Error(`Element with text "${text}" not found`)
      return el
    },
    getByTestId(id) {
      const el = container.querySelector(`[data-testid="${id}"]`)
      if (!el) throw new Error(`Element with testid "${id}" not found`)
      return el
    },
    queryByText(text) {
      return Array.from(container.querySelectorAll('*'))
        .find(el => el.textContent?.trim() === text) ?? null
    },
    async click(element) {
      element.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await Promise.resolve()
    },
    async type(element, text) {
      const input = element as HTMLInputElement
      input.value = text
      input.dispatchEvent(new Event('input', { bubbles: true }))
      await Promise.resolve()
    },
    unmount() {
      document.body.removeChild(container)
    },
  }
}

// mockServerFunction — replace a server function with a test double.
// The mock receives the same arguments and can return a controlled value.
export function mockServerFunction<T extends (...args: unknown[]) => Promise<unknown>>(
  fn: T,
  implementation: (...args: Parameters<T>) => ReturnType<T>
): { mock: T; calls: Parameters<T>[] } {
  const calls: Parameters<T>[] = []
  const mock = ((...args: Parameters<T>) => {
    calls.push(args)
    return implementation(...args)
  }) as T
  return { mock, calls }
}

// mockModule — replace an entire module import in tests.
// Useful for mocking forge:data or forge:auth in component tests.
export function mockModule(
  moduleName: string,
  factory: () => Record<string, unknown>
): void {
  // TODO: Hook into the module resolution system to intercept the import
}

// snapshot — take or compare a component snapshot.
export async function snapshot(component: unknown, props?: Record<string, unknown>): Promise<string> {
  const result = await renderComponent(component, props)
  return result.container.innerHTML
}
