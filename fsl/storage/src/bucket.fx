// Storage.configure() — returns a configured storage instance.
//
// The storage instance is a thin wrapper around the chosen adapter.
// All operations are server-only; the client receives only URLs.

"use module server"

import { FileRef } from './file.fx'
import type { StorageConfig, UploadOptions, PresignedUrlOptions } from './types'

class StorageInstance {
  private config: StorageConfig

  constructor(config: StorageConfig) {
    this.config = config
  }

  async upload(
    prefix: string,
    file: File | Uint8Array | ReadableStream,
    options?: UploadOptions
  ): Promise<FileRef> {
    const key = `${prefix}${options?.filename ?? crypto.randomUUID()}`
    const contentType = options?.contentType ?? (file instanceof File ? file.type : 'application/octet-stream')

    // TODO: Stream file to the configured adapter
    return new FileRef({
      key,
      size: file instanceof File ? file.size : 0,
      contentType,
      publicUrl: this.publicUrlFor(key),
      metadata: options?.metadata,
    })
  }

  async download(key: string): Promise<FileRef> {
    // TODO: Fetch file metadata from the adapter, return FileRef
    return new FileRef({
      key,
      size: 0,
      contentType: 'application/octet-stream',
      publicUrl: this.publicUrlFor(key),
    })
  }

  async presignedUrl(
    prefix: string,
    options?: PresignedUrlOptions
  ): Promise<{ url: string; key: string; expiresAt: Date }> {
    const key = `${prefix}${crypto.randomUUID()}`
    const expiresIn = options?.expiresIn ?? 300

    // TODO: Generate presigned URL via the adapter
    return {
      url: '',
      key,
      expiresAt: new Date(Date.now() + expiresIn * 1000),
    }
  }

  async delete(key: string): Promise<boolean> {
    // TODO: Delete file via the adapter
    return false
  }

  async list(prefix: string): Promise<FileRef[]> {
    // TODO: List files under prefix via the adapter
    return []
  }

  private publicUrlFor(key: string): string {
    const { adapter } = this.config
    if (adapter.type === 'local') {
      return `/storage/${key}`
    }
    if (adapter.type === 's3') {
      return `https://${adapter.bucket}.s3.${adapter.region}.amazonaws.com/${key}`
    }
    if (adapter.type === 'r2') {
      return `${adapter.publicUrl}/${key}`
    }
    return key
  }
}

export const Storage = {
  configure(config: StorageConfig): StorageInstance {
    return new StorageInstance(config)
  },
}
