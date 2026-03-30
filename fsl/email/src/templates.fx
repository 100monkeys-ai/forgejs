// Built-in email template primitives.
//
// Email HTML is hostile: no CSS classes, no custom fonts, nested tables,
// inline styles. These components handle the boilerplate so application
// email templates can focus on content.
//
// All components in this file carry "use module email" which tells the
// compiler they produce email-safe HTML, not browser DOM output.

"use module email"

export component EmailLayout({ children, preheader }: {
  children: unknown
  preheader?: string
}) {
  return (
    <html>
      <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title></title>
      </head>
      <body style="margin: 0; padding: 0; background-color: #f4f4f5; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;">
        {preheader && (
          <div style="display: none; max-height: 0; overflow: hidden; mso-hide: all;">
            {preheader}
          </div>
        )}
        <table role="presentation" cellpadding="0" cellspacing="0" width="100%">
          <tr>
            <td align="center" style="padding: 40px 0;">
              <table role="presentation" cellpadding="0" cellspacing="0" width="600" style="background-color: #ffffff; border-radius: 8px; padding: 40px;">
                <tr>
                  <td>{children}</td>
                </tr>
              </table>
            </td>
          </tr>
        </table>
      </body>
    </html>
  )
}

export component EmailButton({ href, children }: { href: string; children: unknown }) {
  return (
    <table role="presentation" cellpadding="0" cellspacing="0">
      <tr>
        <td style="border-radius: 6px; background-color: #0f172a;">
          <a
            href={href}
            style="display: inline-block; padding: 12px 24px; color: #ffffff; font-size: 14px; font-weight: 600; text-decoration: none;"
          >
            {children}
          </a>
        </td>
      </tr>
    </table>
  )
}

export component EmailText({ children, muted }: { children: unknown; muted?: boolean }) {
  return (
    <p style={`margin: 0 0 16px; font-size: 14px; line-height: 1.6; color: ${muted ? '#6b7280' : '#111827'};`}>
      {children}
    </p>
  )
}
