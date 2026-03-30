export interface RouteDefinition {
  path: string
  component: () => Promise<{ default: unknown }>
  auth?: boolean
  layout?: () => Promise<{ default: unknown }>
}

export interface RouterConfig {
  routes: RouteDefinition[]
  basePath?: string
}

export type Params = Record<string, string>
