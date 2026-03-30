// FileRef — a reference to a stored file.
//
// Returned by upload() and download(). Provides access to the file's
// metadata and transform pipeline.

"use module server"

import type { FileRecord, ImageTransform } from './types'

export class FileRef {
  readonly key: string
  readonly size: number
  readonly contentType: string
  readonly publicUrl: string
  readonly metadata: Record<string, string>

  constructor(record: FileRecord) {
    this.key = record.key
    this.size = record.size
    this.contentType = record.contentType
    this.publicUrl = record.publicUrl
    this.metadata = record.metadata ?? {}
  }

  // transform() — apply Rust-native image transforms.
  // Returns a new FileRef pointing at the transformed output.
  async transform(ops: ImageTransform): Promise<FileRef> {
    // TODO: Pass transform spec to Rust image processor via FFI
    return this
  }

  // toResponse() — stream the file as an HTTP Response.
  toResponse(): Response {
    // TODO: Return a streaming Response from the storage adapter
    return new Response(null, { status: 200 })
  }
}
