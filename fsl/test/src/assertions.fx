// expect() — assertion builder.
//
// Follows the Jest-compatible assertion API. The Forge compiler validates
// assertion calls at build time and can provide improved error messages
// by inlining the source expression text.

"use module test"

class Assertion {
  private value: unknown
  private negated: boolean = false

  constructor(value: unknown) {
    this.value = value
  }

  get not(): this {
    this.negated = !this.negated
    return this
  }

  private assert(condition: boolean, message: string): void {
    const passes = this.negated ? !condition : condition
    if (!passes) {
      throw new Error(this.negated ? `Expected not: ${message}` : message)
    }
  }

  toBe(expected: unknown): void {
    this.assert(
      Object.is(this.value, expected),
      `Expected ${String(this.value)} to be ${String(expected)}`
    )
  }

  toEqual(expected: unknown): void {
    this.assert(
      JSON.stringify(this.value) === JSON.stringify(expected),
      `Expected deep equality`
    )
  }

  toBeDefined(): void {
    this.assert(this.value !== undefined, `Expected value to be defined`)
  }

  toBeNull(): void {
    this.assert(this.value === null, `Expected value to be null`)
  }

  toBeTruthy(): void {
    this.assert(Boolean(this.value), `Expected value to be truthy`)
  }

  toBeFalsy(): void {
    this.assert(!Boolean(this.value), `Expected value to be falsy`)
  }

  toHaveLength(length: number): void {
    const actual = (this.value as { length: number }).length
    this.assert(actual === length, `Expected length ${actual} to be ${length}`)
  }

  toContain(item: unknown): void {
    const arr = this.value as unknown[]
    this.assert(arr.includes(item), `Expected array to contain ${String(item)}`)
  }

  toThrow(message?: string): void {
    let threw = false
    try {
      if (typeof this.value === 'function') this.value()
    } catch (e) {
      threw = true
      if (message) {
        this.assert(
          (e as Error).message.includes(message),
          `Expected error message to contain "${message}"`
        )
        return
      }
    }
    this.assert(threw, `Expected function to throw`)
  }

  resolves = {
    toBe: async (expected: unknown) => {
      const resolved = await (this.value as Promise<unknown>)
      new Assertion(resolved).toBe(expected)
    },
    toEqual: async (expected: unknown) => {
      const resolved = await (this.value as Promise<unknown>)
      new Assertion(resolved).toEqual(expected)
    },
  }

  rejects = {
    toThrow: async (message?: string) => {
      try {
        await (this.value as Promise<unknown>)
        this.assert(false, `Expected promise to reject`)
      } catch (e) {
        if (message) {
          this.assert(
            (e as Error).message.includes(message),
            `Expected rejection message to contain "${message}"`
          )
        }
      }
    },
  }
}

export function expect(value: unknown): Assertion {
  return new Assertion(value)
}
