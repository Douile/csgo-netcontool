use std::collections::VecDeque;
use std::pin::Pin;

use bytes::BytesMut;
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::constants::{BUF_SIZE, NEWLINE};

#[derive(Debug)]
pub struct Buffer {
    data: BytesMut,
    start: usize,
    end: usize,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            data: BytesMut::with_capacity(BUF_SIZE),
            start: 0,
            end: 0,
        }
    }
}

pub struct LineReader<T: AsyncRead + Send> {
    inner: Pin<Box<T>>,
    buffers: VecDeque<Buffer>,
}

impl<T: AsyncRead + Send> LineReader<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: Box::pin(inner),
            buffers: VecDeque::default(),
        }
    }

    async fn read_buffer(&mut self) -> tokio::io::Result<()> {
        loop {
            let mut buffer = Buffer::default();
            let end = self.inner.read_buf(&mut buffer.data).await?;
            if end > 0 {
                buffer.end = end;
                self.buffers.push_back(buffer);
                break;
            }
        }
        Ok(())
    }

    pub async fn read_line(&mut self) -> tokio::io::Result<String> {
        let mut builder = String::new();
        loop {
            if self.buffers.is_empty() {
                self.read_buffer().await?;
            }
            let buffer = self.buffers.front_mut().unwrap();
            let mut p = buffer.start;
            let mut end = None;
            while p < buffer.end {
                if buffer.data[p] == NEWLINE {
                    end = Some(p);
                    break;
                }
                p += 1;
            }
            // If we haven't finished buffer yet
            if let Some(end) = end {
                let res = String::from_utf8_lossy(&buffer.data[buffer.start..end]);
                buffer.start = end + 1;
                builder.push_str(&res);
                return Ok(builder);
            }
            // Read next buffer
            let buffer = self.buffers.pop_front().unwrap();
            let res = String::from_utf8_lossy(&buffer.data[buffer.start..buffer.end]);
            builder.push_str(&res);
        }
    }
}
