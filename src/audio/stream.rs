//! Async stream handling — bridges async HTTP streams to sync readers.
//!
//! This module provides utilities for streaming audio data from HTTP
//! responses into rodio's synchronous decoder pipeline.
//!
//! Phase 1 (MVP): Download full buffer, then decode.
//! Phase 2: Implement a ring-buffer bridge for true streaming playback.

use bytes::Bytes;
use futures::StreamExt;
use std::io::{self, Read, Cursor};

/// A buffered stream reader that downloads audio data in chunks.
///
/// For Phase 2, this will be replaced with a ring-buffer approach
/// that feeds rodio as data arrives, enabling low-latency playback.
pub struct StreamBuffer {
    data: Cursor<Bytes>,
}

impl StreamBuffer {
    /// Create a buffer from already-downloaded bytes.
    pub fn from_bytes(bytes: Bytes) -> Self {
        Self {
            data: Cursor::new(bytes),
        }
    }

    /// Download the full response body and create a buffer.
    pub async fn from_url(url: &str) -> Result<Self, reqwest::Error> {
        let resp = reqwest::get(url).await?;
        let bytes = resp.bytes().await?;
        Ok(Self::from_bytes(bytes))
    }

    /// Get the total size of the buffer.
    pub fn len(&self) -> usize {
        self.data.get_ref().len()
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Read for StreamBuffer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.data.read(buf)
    }
}

/// Download audio data in chunks, reporting progress via callback.
pub async fn download_with_progress<F>(
    url: &str,
    on_progress: F,
) -> Result<Bytes, reqwest::Error>
where
    F: Fn(usize, Option<usize>),
{
    let resp = reqwest::get(url).await?;
    let total = resp.content_length().map(|l| l as usize);
    let mut stream = resp.bytes_stream();
    let mut downloaded = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        downloaded.extend_from_slice(&chunk);
        on_progress(downloaded.len(), total);
    }

    Ok(Bytes::from(downloaded))
}
