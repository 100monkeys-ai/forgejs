// Schema — table and column definition DSL.
//
// Tables defined with Schema.table() are used to:
// 1. Generate SQL migrations via `forge migrate`
// 2. Produce TypeScript types for query results at compile time
// 3. Build the query builder's type-safe API surface
//
// Schema definitions live in app/schema.fx and are imported both by the
// compiler (for codegen) and by server functions (for runtime queries).

"use module server"

export interface ColumnDef {
  type: 'id' | 'text' | 'integer' | 'boolean' | 'timestamp' | 'json'
  nullable: boolean
  unique: boolean
  default?: string | number | boolean | null
  references?: { table: string; column: string }
}

export interface TableDef {
  name: string
  columns: Record<string, ColumnDef>
}

function col(type: ColumnDef['type']): ColumnBuilder {
  return new ColumnBuilder({ type, nullable: true, unique: false })
}

class ColumnBuilder {
  private def: ColumnDef

  constructor(def: ColumnDef) {
    this.def = { ...def }
  }

  notNull(): this {
    this.def.nullable = false
    return this
  }

  unique(): this {
    this.def.unique = true
    return this
  }

  default(value: string | number | boolean): this {
    this.def.default = value
    return this
  }

  references(table: string, column: string = 'id'): this {
    this.def.references = { table, column }
    return this
  }

  build(): ColumnDef {
    return this.def
  }
}

export const Schema = {
  table(name: string, columns: Record<string, ColumnBuilder>): TableDef {
    const resolved: Record<string, ColumnDef> = {}
    for (const [key, builder] of Object.entries(columns)) {
      resolved[key] = builder.build()
    }
    return { name, columns: resolved }
  },

  id() {
    return col('id').notNull()
  },

  text() {
    return col('text')
  },

  integer() {
    return col('integer')
  },

  boolean() {
    return col('boolean')
  },

  timestamp() {
    return col('timestamp')
  },

  json() {
    return col('json')
  },
}
