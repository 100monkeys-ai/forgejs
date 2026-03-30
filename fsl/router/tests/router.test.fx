// forge:router tests
import { describe, it, expect } from 'forge:test'
import { Router } from '../src/router.fx'

describe('Router.define', () => {
  it('returns the route definitions', () => {
    const routes = Router.define([
      { path: '/', component: async () => ({ default: {} }) }
    ])
    expect(routes).toHaveLength(1)
    expect(routes[0].path).toBe('/')
  })
})
