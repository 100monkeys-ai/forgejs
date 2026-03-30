export interface ColumnDefinition {
  type: 'id' | 'text' | 'integer' | 'boolean' | 'timestamp' | 'json'
  nullable: boolean
  unique: boolean
  default?: string | number | boolean | null
  references?: { table: string; column: string }
}

export interface TableDefinition {
  name: string
  columns: Record<string, ColumnDefinition>
}

export interface QueryBuilder<T> {
  where(conditions: Partial<T>): QueryBuilder<T>
  orderBy(column: keyof T, dir?: 'asc' | 'desc'): QueryBuilder<T>
  limit(n: number): QueryBuilder<T>
  offset(n: number): QueryBuilder<T>
  all(): Promise<T[]>
  first(): Promise<T | null>
  count(): Promise<number>
}

export interface DbAdapter {
  name: 'sqlite' | 'postgresql' | 'libsql'
  connectionString: string
}

export interface MigrationPlan {
  version: number
  name: string
  up: string
  down: string
  appliedAt?: Date
}
