// Test runner primitives — describe, it, beforeEach, etc.
//
// The Forge test runner is implemented in Rust and drives these primitives.
// At build time, forge:test imports are detected and the test harness is
// injected. Test files are not bundled into production output.

"use module test"

type TestFn = () => void | Promise<void>

interface TestSuite {
  name: string
  tests: TestCase[]
  beforeEach?: TestFn
  afterEach?: TestFn
  beforeAll?: TestFn
  afterAll?: TestFn
}

interface TestCase {
  name: string
  fn: TestFn
  only?: boolean
  skip?: boolean
}

// These are stubs — the Rust test runner replaces them at test execution time.
export function describe(name: string, fn: () => void): void {
  // Runner registers the suite
  fn()
}

export function it(name: string, fn: TestFn): void {
  // Runner registers the test case
}

export const test = it

export function beforeEach(fn: TestFn): void {}
export function afterEach(fn: TestFn): void {}
export function beforeAll(fn: TestFn): void {}
export function afterAll(fn: TestFn): void {}
