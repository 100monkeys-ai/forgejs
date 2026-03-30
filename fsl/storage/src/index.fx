// forge:storage — Forge Standard Library file storage module
//
// Transport-agnostic file storage with S3, Cloudflare R2, and local
// filesystem adapters. Image transforms (resize, crop, format convert)
// are handled natively in Rust — no Sharp, no libvips dependency.
//
// Example — app/storage.fx:
//   import { Storage } from 'forge:storage'
//   export const storage = Storage.configure({
//     adapter: { type: 's3', bucket: env.S3_BUCKET, region: env.AWS_REGION },
//   })
//
// Example — upload from a server function:
//   import { storage } from 'app/storage.fx'
//   const file = await storage.upload('avatars/', formData.get('avatar') as File)
//   return { url: file.publicUrl }
//
// Example — client-side direct upload with presigned URL:
//   import { storage } from 'app/storage.fx'
//   const { url, key } = await storage.presignedUrl('uploads/', { expiresIn: 300 })
//   await fetch(url, { method: 'PUT', body: file })

export { Storage } from './bucket.fx'
export { FileRef } from './file.fx'
export type {
  StorageConfig,
  StorageAdapter,
  FileRecord,
  UploadOptions,
  PresignedUrlOptions,
  ImageTransform,
  S3Config,
  R2Config,
  LocalConfig,
} from './types'
