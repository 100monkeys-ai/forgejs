// sendEmail() — dispatch a transactional email.
//
// The template is a .fx component with "use module email" at the top.
// The compiler renders it to static HTML at build time when props are
// statically known, or at request time for dynamic content.

"use module server"

import type { EmailMessage, EmailAdapter } from './types'

let adapter: EmailAdapter | null = null

export function configureEmail(config: EmailAdapter): void {
  adapter = config
}

export async function sendEmail<Props extends Record<string, unknown>>(message: {
  to: string | string[]
  from?: string
  replyTo?: string
  subject: string
  template: (props: Props) => unknown
  props: Props
  attachments?: Array<{ filename: string; content: Uint8Array; contentType: string }>
}): Promise<{ messageId: string }> {
  if (!adapter) {
    throw new Error(
      'forge:email is not configured. Call configureEmail() in app/email.fx before sending.'
    )
  }

  // Render the template component to HTML
  const html = renderEmailTemplate(message.template, message.props)

  const emailMessage: EmailMessage = {
    to: Array.isArray(message.to) ? message.to : [message.to],
    from: message.from ?? adapter.defaultFrom,
    replyTo: message.replyTo,
    subject: message.subject,
    html,
    attachments: message.attachments,
  }

  return adapter.send(emailMessage)
}

function renderEmailTemplate<Props extends Record<string, unknown>>(
  template: (props: Props) => unknown,
  props: Props
): string {
  // TODO: Invoke the Forge server-side renderer in email mode
  // Email rendering strips client-only primitives ($signal, $effect, etc.)
  return ''
}
