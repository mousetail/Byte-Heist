use std::io::{PipeWriter, Write};

use nix::fcntl::OFlag;
use tokio::io::unix::AsyncFd;

pub struct LimitedAsyncPipeWriter {
    inner: AsyncFd<PipeWriter>,
    data: Vec<u8>,
    ptr: usize,
}

impl Unpin for LimitedAsyncPipeWriter {}

pub struct LimitedAsyncPipeWriterOutput {
    #[allow(unused)]
    prematurely_closed: bool,
}

impl LimitedAsyncPipeWriter {
    pub fn new(inner: PipeWriter, data: Vec<u8>) -> Result<Self, std::io::Error> {
        // Set the pipe writer in "non-blocking" mode
        let args: OFlag =
            OFlag::from_bits(nix::fcntl::fcntl(&inner, nix::fcntl::FcntlArg::F_GETFL)?)
                .expect("Expected valid file flags");
        nix::fcntl::fcntl(
            &inner,
            nix::fcntl::FcntlArg::F_SETFL(args | OFlag::O_NONBLOCK),
        )?;

        Ok(Self {
            inner: AsyncFd::new(inner)?,
            data,
            ptr: 0,
        })
    }
}

impl Future for LimitedAsyncPipeWriter {
    type Output = std::io::Result<LimitedAsyncPipeWriterOutput>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let Self {
            ref mut inner,
            ref mut data,
            ref mut ptr,
        } = *self.as_mut();

        while let std::task::Poll::Ready(guard) = inner.poll_write_ready_mut(cx) {
            let mut guard = guard?;

            match guard.get_inner_mut().write(&data[*ptr..]) {
                Ok(0) => {
                    return std::task::Poll::Ready(Ok(LimitedAsyncPipeWriterOutput {
                        prematurely_closed: true,
                    }));
                }
                Ok(n) => {
                    *ptr += n;

                    if *ptr >= data.len() {
                        return std::task::Poll::Ready(Ok(LimitedAsyncPipeWriterOutput {
                            prematurely_closed: false,
                        }));
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    guard.clear_ready();
                }
                Err(e) => {
                    return std::task::Poll::Ready(Err(e));
                }
            }
        }

        std::task::Poll::Pending
    }
}
