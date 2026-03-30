// forge:data — Forge Standard Library database module
//
// Provides a schema-first database layer with type-safe queries, automatic
// migration generation, and reactive $db signals for UI data binding.
//
// Example:
//   import { Schema, db } from 'forge:data'
//
//   export const Users = Schema.table('users', {
//     id: Schema.id(),
//     name: Schema.text().notNull(),
//     email: Schema.text().unique().notNull(),
//     createdAt: Schema.timestamp().default('now'),
//   })
//
//   // In a server function:
//   const users = await db.query(Users).where({ name: 'Alice' }).all()

export { Schema } from './schema.fx'
export { db, $db } from './query.fx'
export type {
  TableDefinition,
  ColumnDefinition,
  QueryBuilder,
  DbAdapter,
  MigrationPlan,
} from './types'
