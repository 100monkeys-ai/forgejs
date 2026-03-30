// forge:test — Forge Standard Library test runner module
//
// Provides the standard test API (`describe`, `it`, `expect`) plus
// Forge-specific helpers for testing components and server functions.
//
// Tests live in files ending in `.test.fx` or `.spec.fx`. Run them with:
//   forge test
//   forge test --watch
//   forge test --coverage
//
// Example:
//   import { describe, it, expect, renderComponent } from 'forge:test'
//   import Counter from '../src/Counter.fx'
//
//   describe('Counter', () => {
//     it('increments on click', async () => {
//       const { getByText, click } = await renderComponent(<Counter initial={0} />)
//       click(getByText('+'))
//       expect(getByText('1')).toBeDefined()
//     })
//   })

export { describe, it, test, beforeEach, afterEach, beforeAll, afterAll } from './runner.fx'
export { expect } from './assertions.fx'
export { renderComponent, mockServerFunction, mockModule } from './mocks.fx'
export { snapshot } from './mocks.fx'
export type {
  TestSuite,
  TestCase,
  Assertion,
  RenderResult,
  MockServerFunction,
} from './types'
