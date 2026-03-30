// Navigation utilities: programmatic navigation and route parameter access.

"use module client"

export function navigate(path: string, options?: { replace?: boolean }) {
  if (options?.replace) {
    window.history.replaceState(null, '', path)
  } else {
    window.history.pushState(null, '', path)
  }
  // TODO: Notify router of navigation
}

export function useParams<T extends Record<string, string>>(): T {
  // TODO: Return current route parameters from router context
  return {} as T
}

export function useLocation(): string {
  return window.location.pathname
}
