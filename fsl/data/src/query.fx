// db and $db — the query interface.
//
// `db` is the standard async query API for use in server functions.
// `$db` is a reactive signal wrapper that re-runs the query when its
// dependencies change, powering live-updating UI components.
//
// Both are server-only. The compiler rejects any import of db or $db
// from a client module.

"use module server"

import type { TableDef } from './schema.fx'

export interface QueryOptions<T> {
  where?: Partial<T>
  orderBy?: keyof T
  orderDir?: 'asc' | 'desc'
  limit?: number
  offset?: number
}

async function executeAdapterQuery(operation: string, tableName: string, payload?: any): Promise<any> {
  // TODO: Execute via the bound DbAdapter
  console.log(`[fsl:data] Stub ${operation} into ${tableName}`, payload ?? '')
  return null
}

class QueryBuilder<T> {
  private tableDef: TableDef
  private options: QueryOptions<T> = {}

  constructor(table: TableDef) {
    this.tableDef = table
  }

  where(conditions: Partial<T>): this {
    this.options.where = conditions
    return this
  }

  orderBy(column: keyof T, dir: 'asc' | 'desc' = 'asc'): this {
    this.options.orderBy = column
    this.options.orderDir = dir
    return this
  }

  limit(n: number): this {
    this.options.limit = n
    return this
  }

  offset(n: number): this {
    this.options.offset = n
    return this
  }

  async all(): Promise<T[]> {
    await executeAdapterQuery('SELECT', this.tableDef.name, this.options)
    return []
  }

  async first(): Promise<T | null> {
    const results = await this.limit(1).all()
    return results[0] ?? null
  }

  async count(): Promise<number> {
    await executeAdapterQuery('COUNT', this.tableDef.name, this.options)
    return 0
  }
}

export const db = {
  query<T>(table: TableDef): QueryBuilder<T> {
    return new QueryBuilder<T>(table)
  },

  async insert<T>(table: TableDef, data: Omit<T, 'id'>): Promise<T> {
    await executeAdapterQuery('INSERT', table.name, data)
    return { id: 1, ...data } as unknown as T
  },

  async update<T>(table: TableDef, id: string | number, data: Partial<T>): Promise<T | null> {
    await executeAdapterQuery('UPDATE', table.name, { id, ...data })
    return null
  },

  async delete(table: TableDef, id: string | number): Promise<boolean> {
    await executeAdapterQuery('DELETE', table.name, { id })
    return false
  },
}

// $db — reactive signal wrapper around db.query()
// When used inside a component or $derived, re-evaluates when the signal
// graph is invalidated (e.g., after a mutation).
export function $db<T>(table: TableDef, options?: QueryOptions<T>) {
  // TODO: Integrate with the TC39 Signals runtime
  // Returns a Signal<T[]> that resolves asynchronously
  return {
    get value(): T[] {
      return []
    }
  }
}
