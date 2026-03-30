export interface FileRecord {
  key: string
  size: number
  contentType: string
  publicUrl: string
  metadata?: Record<string, string>
}

export interface UploadOptions {
  filename?: string
  contentType?: string
  metadata?: Record<string, string>
  maxSizeBytes?: number
}

export interface PresignedUrlOptions {
  expiresIn?: number
  contentType?: string
  maxSizeBytes?: number
}

export interface S3Config {
  type: 's3'
  bucket: string
  region: string
  accessKeyId?: string
  secretAccessKey?: string
  endpoint?: string
}

export interface R2Config {
  type: 'r2'
  bucket: string
  accountId: string
  accessKeyId: string
  secretAccessKey: string
  publicUrl: string
}

export interface LocalConfig {
  type: 'local'
  directory: string
  servePrefix?: string
}

export type StorageAdapter = S3Config | R2Config | LocalConfig

export interface StorageConfig {
  adapter: StorageAdapter
}

export interface ImageTransform {
  resize?: { width?: number; height?: number; fit?: 'cover' | 'contain' | 'fill' }
  format?: 'webp' | 'avif' | 'jpeg' | 'png'
  quality?: number
  blur?: number
}
