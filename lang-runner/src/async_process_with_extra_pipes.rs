//! Currently, it's impossible in the
//! Command API (both Tokio, async_process from smol, and STD) to pass custom
//! file descriptors to a child process except stdin, stdout, and stderr
//! However, bwrap makes quite extensive use of custom file descriptors to
//! get around the max argument size limit and for bind mounts.
//!
//! This, I wrote this custom implementation of process using the "nix" crate.
//! However, all of this should be removed when https://github.com/rust-lang/rust/issues/144989
//! is stabalized.

use std::{
    collections::HashMap,
    ffi::CStr,
    io::{PipeReader, Read, Write},
    os::fd::AsRawFd,
    pin::Pin,
    sync::{Arc, OnceLock},
    task::{Context, Poll},
};

use futures_util::FutureExt;
use nix::{
    errno::Errno,
    spawn::{PosixSpawnAttr, PosixSpawnFileActions, posix_spawn},
    sys::{
        signal::{Signal, kill},
        wait::{WaitPidFlag, WaitStatus},
    },
    unistd::Pid,
};

/// A future that waits for a child process with a given PID to complete
struct AsyncChild {
    child: Pid,
    thread: Option<std::thread::JoinHandle<()>>,
    result: Arc<OnceLock<Result<WaitStatus, Errno>>>,
    exited: bool,
}

impl AsyncChild {
    fn new(child: Pid) -> Self {
        AsyncChild {
            child,
            thread: None,
            exited: false,
            result: Arc::new(OnceLock::new()),
        }
    }

    fn understand_wait_status(status: WaitStatus) -> Option<SignalOrStatus> {
        match status {
            WaitStatus::Exited(_pid, status) => {
                if status < 129 {
                    Some(SignalOrStatus::Status(status))
                } else {
                    Some(
                        (status - 128)
                            .try_into()
                            .map(SignalOrStatus::Signal)
                            .unwrap_or(SignalOrStatus::Status(status)),
                    )
                }
            }
            WaitStatus::Signaled(_pid, signal, _core_dump_avalible) => {
                Some(SignalOrStatus::Signal(signal))
            }
            _ => None,
        }
    }
}

pub enum SignalOrStatus {
    Signal(Signal),
    Status(i32),
}

impl Future for AsyncChild {
    type Output = Result<SignalOrStatus, std::io::Error>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.thread.take() {
            None => {
                let status = nix::sys::wait::waitpid(self.child, Some(WaitPidFlag::WNOHANG));
                let exit_code = match status {
                    Ok(e) => Self::understand_wait_status(e),
                    Err(err) => return Poll::Ready(Err(err.into())),
                };
                match exit_code {
                    Some(e) => {
                        self.exited = true;
                        eprintln!("Child exited before first poll");
                        Poll::Ready(Ok(e))
                    }
                    None => {
                        let child = self.child;
                        let waker = cx.waker().clone();
                        let result = self.result.clone();
                        self.thread = Some(std::thread::spawn(move || {
                            let status = nix::sys::wait::waitpid(child, None);
                            result
                                .set(status)
                                .expect("Exit status was already previously set");
                            eprintln!("Child process exited normally");
                            waker.wake();
                        }));
                        Poll::Pending
                    }
                }
            }
            Some(e) => {
                let status = self.result.get();
                if let Some(status) = status.cloned() {
                    self.exited = true;
                    e.join().expect("Watcher thread panicked");
                    let exit_code = match status {
                        Ok(e) => Self::understand_wait_status(e),
                        Err(err) => return Poll::Ready(Err(err.into())),
                    };
                    Poll::Ready(Ok(
                        exit_code.unwrap_or(SignalOrStatus::Signal(Signal::SIGUSR1))
                    ))
                } else {
                    eprintln!("Process polled but thread has not yet finished");
                    self.thread = Some(e);
                    Poll::Pending
                }
            }
        }
    }
}

impl Drop for AsyncChild {
    fn drop(&mut self) {
        if !self.exited {
            eprintln!("Child timed out, killing child...");
            if let Err(e) = kill(self.child, Signal::SIGTERM) {
                eprintln!("Error killing child: {e:?}")
            }
            // clean up zombie process if the thread was not started yet
            if self.thread.is_none()
                && !self.exited
                && let Err(e) = nix::sys::wait::waitpid(self.child, None)
            {
                eprintln!("Error harvesting child' corpse: {e:?}");
            };
        }
    }
}

pub struct ChildOutput {
    pub result: SignalOrStatus,
    pub outputs: HashMap<i32, String>,
}

pub struct OutputChild {
    process: AsyncChild,
    pipes: HashMap<i32, PipeReader>,
}

impl Future for OutputChild {
    type Output = Result<ChildOutput, std::io::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.process.poll_unpin(cx) {
            Poll::Ready(result) => Poll::Ready(result.and_then(|exit_status| {
                let pipes = std::mem::take(&mut self.pipes);
                let pipes = pipes
                    .into_iter()
                    .map(|(key, mut value)| {
                        let mut str = String::new();
                        value.read_to_string(&mut str)?;
                        Ok((key, str))
                    })
                    .collect::<Result<HashMap<_, _>, std::io::Error>>()?;

                Ok(ChildOutput {
                    result: exit_status,
                    outputs: pipes,
                })
            })),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// This implements only a small subset of the command API
/// It also only "async" for the purpose of waiting for the child to exit
/// It will execute blocking reads from the pipes when the child exits
pub struct AsyncProcessWithCustomPipes<'a> {
    command: &'a CStr,
    arguments: Vec<&'a CStr>,
    env: &'a [&'a CStr],

    process_input: HashMap<i32, &'a [u8]>,
    process_output: Vec<i32>,
}

impl<'a> AsyncProcessWithCustomPipes<'a> {
    pub fn new(command: &'a CStr, arguments: Vec<&'a CStr>, env: &'a [&'a CStr]) -> Self {
        Self {
            command,
            arguments,
            env,
            process_input: HashMap::new(),
            process_output: vec![],
        }
    }

    pub fn add_input(mut self, fd: i32, data: &'a [u8]) -> Self {
        self.process_input.insert(fd, data);
        self
    }

    pub fn add_output(mut self, fd: i32) -> Self {
        self.process_output.push(fd);
        self
    }

    pub fn output(self) -> Result<OutputChild, std::io::Error> {
        let mut file_actions = PosixSpawnFileActions::init()?;
        let attrs = PosixSpawnAttr::init()?;

        let mut readers_to_be_dropped = Vec::with_capacity(self.process_input.len());
        let mut writers_to_be_dropped = Vec::with_capacity(self.process_output.len());

        for (fd, data) in self.process_input {
            let (reader, mut writer) = std::io::pipe()?;
            file_actions.add_dup2(reader.as_raw_fd(), fd)?;
            writer.write_all(data)?;
            readers_to_be_dropped.push(reader);
        }

        let mut readers = HashMap::new();

        for fd in self.process_output {
            let (reader, writer) = std::io::pipe()?;
            file_actions.add_dup2(writer.as_raw_fd(), fd)?;
            readers.insert(fd, reader);

            writers_to_be_dropped.push(writer);
        }

        let process = posix_spawn(
            self.command,
            &file_actions,
            &attrs,
            &self.arguments,
            self.env,
        )?;

        Ok(OutputChild {
            process: AsyncChild::new(process),
            pipes: readers,
        })
    }
}
