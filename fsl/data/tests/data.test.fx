// forge:data tests
import { describe, it, expect } from 'forge:test'
import { Schema, db } from '../src/index.fx'

const Posts = Schema.table('posts', {
  id: Schema.id(),
  title: Schema.text().notNull(),
  body: Schema.text().notNull(),
  published: Schema.boolean().default(false),
  createdAt: Schema.timestamp().default('now'),
})

describe('Schema.table', () => {
  it('defines a table with columns', () => {
    expect(Posts.name).toBe('posts')
    expect(Object.keys(Posts.columns)).toContain('title')
    expect(Posts.columns.title.nullable).toBe(false)
  })

  it('applies column modifiers', () => {
    expect(Posts.columns.id.type).toBe('id')
    expect(Posts.columns.published.default).toBe(false)
  })
})

describe('db.query', () => {
  it('returns a QueryBuilder for a table', () => {
    const qb = db.query(Posts)
    expect(qb).toBeDefined()
    expect(typeof qb.where).toBe('function')
    expect(typeof qb.all).toBe('function')
  })
})
