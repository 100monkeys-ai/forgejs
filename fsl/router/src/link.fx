// <Link> — client-side navigation without a full page reload.
// Wraps an <a> tag and intercepts clicks to update the URL via the History API.

"use module client"

export component Link({ href, children, ...props }: {
  href: string
  children: unknown
  [key: string]: unknown
}) {
  const handleClick = (e: MouseEvent) => {
    e.preventDefault()
    // TODO: Push to history, update router state
  }

  return <a href={href} onClick={handleClick} {...props}>{children}</a>
}
