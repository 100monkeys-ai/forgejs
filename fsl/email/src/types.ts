export interface EmailMessage {
  to: string[]
  from: string
  replyTo?: string
  subject: string
  html: string
  text?: string
  attachments?: Array<{
    filename: string
    content: Uint8Array
    contentType: string
  }>
}

export interface EmailAdapter {
  defaultFrom: string
  send(message: EmailMessage): Promise<{ messageId: string }>
}

export interface SmtpConfig {
  type: 'smtp'
  host: string
  port: number
  secure: boolean
  auth: { user: string; pass: string }
  defaultFrom: string
}

export interface ResendConfig {
  type: 'resend'
  apiKey: string
  defaultFrom: string
}

export interface PostmarkConfig {
  type: 'postmark'
  serverToken: string
  defaultFrom: string
}
