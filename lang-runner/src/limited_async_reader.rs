use std::io::{PipeReader, Read};

use nix::fcntl::OFlag;
use tokio::io::unix::AsyncFd;

use crate::error::RunProcessError;

pub struct LimitedAsyncPipeReader {
    inner: AsyncFd<PipeReader>,
    max_length: usize,
    truncated: bool,
    buffer: Vec<u8>,
}

impl Unpin for LimitedAsyncPipeReader {}

pub struct LimitedAsyncPipeReaderOutput {
    value: Vec<u8>,
    // Reserved for future use
    #[allow(unused)]
    truncated: bool,
}

impl LimitedAsyncPipeReaderOutput {
    pub fn into_string(self) -> Result<String, RunProcessError> {
        String::from_utf8(self.value).map_err(|_| RunProcessError::InvalidUtf8)
    }
}

const CHUNK_SIZE: usize = 1024;

impl LimitedAsyncPipeReader {
    pub fn new(inner: PipeReader, max_length: usize) -> Result<Self, std::io::Error> {
        // Set the pipe reader in "non-blocking" mode
        let args: OFlag =
            OFlag::from_bits(nix::fcntl::fcntl(&inner, nix::fcntl::FcntlArg::F_GETFL)?)
                .expect("Expected valid file flags");
        nix::fcntl::fcntl(
            &inner,
            nix::fcntl::FcntlArg::F_SETFL(args | OFlag::O_NONBLOCK),
        )?;

        Ok(Self {
            inner: AsyncFd::new(inner)?,
            max_length,
            truncated: false,
            buffer: Vec::with_capacity(CHUNK_SIZE),
        })
    }
}

impl Future for LimitedAsyncPipeReader {
    type Output = std::io::Result<LimitedAsyncPipeReaderOutput>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let Self {
            ref mut inner,
            ref max_length,
            ref mut truncated,
            ref mut buffer,
        } = *self.as_mut();

        while let std::task::Poll::Ready(guard) = inner.poll_read_ready_mut(cx) {
            let mut guard = guard?;
            let original_length = buffer.len();
            buffer.resize(original_length + CHUNK_SIZE, 0);

            match guard.get_inner_mut().read(&mut buffer[original_length..]) {
                Ok(0) => {
                    buffer.truncate(original_length);
                    return std::task::Poll::Ready(Ok(LimitedAsyncPipeReaderOutput {
                        value: std::mem::take(buffer),
                        truncated: *truncated,
                    }));
                }
                Ok(n) => {
                    if original_length + n > *max_length {
                        *truncated = true;
                    }
                    buffer.truncate((*max_length).min(original_length + n));
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    buffer.truncate(original_length);
                    guard.clear_ready();
                }
                Err(e) => {
                    buffer.truncate(original_length);
                    return std::task::Poll::Ready(Err(e));
                }
            }
        }

        std::task::Poll::Pending
    }
}
