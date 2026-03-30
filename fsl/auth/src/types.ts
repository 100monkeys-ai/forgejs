export interface User {
  id: string
  email: string
  name?: string
  role?: string
  createdAt: Date
  updatedAt: Date
}

export interface Session {
  id: string
  userId: string
  createdAt: Date
  expiresAt: Date
}

export interface PermissionRule {
  (context: { user: User; resource?: Record<string, unknown> }): boolean | Promise<boolean>
}

export interface SessionConfig {
  secret: string
  maxAge?: number
  secure?: boolean
  sameSite?: 'strict' | 'lax' | 'none'
}

export interface OAuthConfig {
  clientId: string
  clientSecret: string
  scopes?: string[]
  redirectUri?: string
}

export interface Provider {
  type: 'email' | 'oauth' | 'passkey'
}

export interface AuthConfig {
  session: SessionConfig
  providers: Provider[]
  permissions?: Record<string, PermissionRule>
  loginPath?: string
}
