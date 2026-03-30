// forge:email — Forge Standard Library email module
//
// Transactional email with .fx templates. Email templates use the same
// reactive component syntax as UI components, compiled to HTML at build
// time (not runtime). No template string concatenation, no separate
// template language.
//
// Example — app/emails/WelcomeEmail.fx:
//   "use module email"
//   export default component WelcomeEmail({ name }: { name: string }) {
//     return (
//       <EmailLayout>
//         <h1>Welcome, {name}!</h1>
//         <p>Thanks for signing up.</p>
//       </EmailLayout>
//     )
//   }
//
// Example — in a server function:
//   import { sendEmail } from 'forge:email'
//   import WelcomeEmail from 'app/emails/WelcomeEmail.fx'
//
//   await sendEmail({
//     to: user.email,
//     subject: 'Welcome!',
//     template: WelcomeEmail,
//     props: { name: user.name },
//   })

export { sendEmail, configureEmail } from './send.fx'
export { EmailLayout, EmailButton, EmailText } from './templates.fx'
export type {
  EmailMessage,
  EmailAdapter,
  SmtpConfig,
  ResendConfig,
  PostmarkConfig,
} from './types'
