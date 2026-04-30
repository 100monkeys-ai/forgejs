// forge:test self-tests
// These tests verify the test runner's own assertion API.
import { describe, it, expect, renderComponent } from '../src/index.fx'

describe('expect().toBe', () => {
  it('passes for identical primitives', () => {
    expect(1).toBe(1)
    expect('hello').toBe('hello')
    expect(true).toBe(true)
    expect(null).toBe(null)
  })

  it('fails via .not for mismatched values', () => {
    expect(1).not.toBe(2)
    expect('a').not.toBe('b')
  })
})

describe('expect().toHaveLength', () => {
  it('checks array length', () => {
    expect([1, 2, 3]).toHaveLength(3)
    expect([]).toHaveLength(0)
  })
})

describe('expect().toThrow', () => {
  it('catches thrown errors', () => {
    expect(() => { throw new Error('boom') }).toThrow('boom')
  })
})

describe('renderComponent', () => {
  it('mounts a component and renders its output', async () => {
    // Dummy component function mimicking compiled Forge output
    const MyComponent = (props: { title?: string } = {}) => {
      const el = document.createElement('div')
      el.textContent = props.title ?? 'Hello World'
      return el
    }

    const { getByText, unmount } = await renderComponent(MyComponent, { title: 'Test Component' })

    expect(getByText('Test Component')).toBeDefined()

    unmount()
  })

  it('mounts a pre-constructed Element directly', async () => {
    const element = document.createElement('p')
    element.textContent = 'Pre-built element'

    const { getByText, unmount } = await renderComponent(element)

    expect(getByText('Pre-built element')).toBeDefined()

    unmount()
  })
})
