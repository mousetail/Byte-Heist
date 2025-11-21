use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ffi::{CStr, CString};
use std::fmt::Write;

use common::TimerType;
use common::langs::{LANGS, Lang};
use nix::libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};
use serde::Serialize;
use tempfile::TempDir;

use crate::async_process_with_extra_pipes::{AsyncProcessWithCustomPipes, SignalOrStatus};
use crate::error::RunProcessError;
use crate::install_lang::get_lang_directory;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunCodeResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_status: i32,
}

pub struct RunLangContext {
    tmp_folder: TempDir,
    compiled_programs: HashMap<String, CString>,
    lang: &'static Lang,
    lang_folder: CString,
    run_command: Vec<CString>,
    compile_command: Option<Vec<CString>>,
}

impl RunLangContext {
    pub async fn new(lang_name: &str, lang_version: &str) -> Result<Self, RunProcessError> {
        let lang = LANGS
            .get(lang_name)
            .ok_or(RunProcessError::NoSuchLanguage)?;
        let lang_folder = CString::new(
            get_lang_directory(lang, lang_version)
                .await?
                .to_str()
                .expect("Expected path to be valid unicode"),
        )
        .expect("lang folder should not have null bytes");

        Ok(RunLangContext {
            tmp_folder: TempDir::new()?,
            compiled_programs: HashMap::new(),
            lang,
            lang_folder,
            run_command: Self::run_substitutions(lang.run_command, lang.extension),
            compile_command: lang
                .compile_command
                .map(|compile_command| Self::run_substitutions(compile_command, lang.extension)),
        })
    }

    fn run_substitutions(command: &[&str], extension: &str) -> Vec<CString> {
        command
            .iter()
            .map(|segment| {
                CString::new(
                    segment
                        .replace("${LANG_LOCATION}", "/lang")
                        .replace("${FILE_LOCATION}", &format!("/code{}", extension))
                        .replace("${OUTPUT_LOCATION}", "/artifact/1"),
                )
                .expect("Expected substitutions to not contain null bytes")
            })
            .collect()
    }

    pub async fn run(
        &mut self,
        code: &str,
        input: Option<&str>,
        sender: &mut tokio::sync::mpsc::Sender<TimerType>,
    ) -> Result<RunCodeResult, RunProcessError> {
        let code_mount = match self.lang.extension {
            "" => Cow::Borrowed(c"/code"),
            e => Cow::Owned(
                CString::new(format!("/code{e}"))
                    .expect("Expected the extension to not contain null bytes"),
            ),
        };

        match &self.compile_command {
            None => {
                let _ = sender.send(TimerType::Run).await;
                eprintln!("Starting run with lang {}", self.lang.display_name);
                let mut sandbox = RunInSandboxBuilder::new(
                    self.lang,
                    &self.lang_folder,
                    self.run_command.as_slice(),
                )
                .mount_string(&code_mount, code.as_bytes());
                if let Some(input) = input {
                    sandbox = sandbox.set_input(input.as_bytes());
                }
                let result = sandbox.run().await;
                eprintln!("Finished run with lang {}", self.lang.display_name);

                let _ = sender.send(TimerType::Judge).await;
                result
            }
            Some(compile_command) => {
                let compiled_programs_length = self.compiled_programs.len();
                let entry = self.compiled_programs.entry(code.to_owned());

                let (folder, stderr) = match entry {
                    Entry::Occupied(ref value) => (value.get(), String::new()),
                    Entry::Vacant(t) => {
                        let _ = sender.send(TimerType::Compile).await;
                        let folder = self
                            .tmp_folder
                            .path()
                            .join(format!("{}", compiled_programs_length));

                        std::fs::create_dir(&folder)?;

                        let folder_cstr = CString::new(
                            folder
                                .to_str()
                                .expect("Expected temp folder to be a valid string"),
                        )
                        .expect("Expected the temp folder to not contain null bytes");
                        let result =
                            RunInSandboxBuilder::new(self.lang, &self.lang_folder, compile_command)
                                .mount_folder(&folder_cstr, c"/artifact")
                                .mount_string(&code_mount, code.as_bytes())
                                .run()
                                .await?;

                        (t.insert(folder_cstr) as &CString, result.stderr)
                    }
                };
                let _ = sender.send(TimerType::Run).await;

                let mut sandbox =
                    RunInSandboxBuilder::new(self.lang, &self.lang_folder, &self.run_command)
                        .mount_ro_folder(folder, c"/artifact");

                if let Some(input) = input {
                    sandbox = sandbox.set_input(input.as_bytes());
                }

                let mut result = sandbox.run().await?;
                let _ = sender.send(TimerType::Judge).await;
                result.stderr.insert_str(0, &stderr);
                Ok(result)
            }
        }
    }
}

struct RunInSandboxBuilder<'a> {
    bubblewrap_args: Vec<u8>,
    process: AsyncProcessWithCustomPipes<'a>,
    pipe_number: i32,
    command: &'a [CString],
}

impl<'a> RunInSandboxBuilder<'a> {
    fn add_args(mut self, args: impl IntoIterator<Item = impl AsRef<CStr>>) -> Self {
        for arg in args {
            self.bubblewrap_args
                .extend_from_slice(arg.as_ref().to_bytes_with_nul());
        }
        self
    }

    fn new(lang: &'static Lang, lang_folder: &CStr, command: &'a [CString]) -> Self {
        let mut result = RunInSandboxBuilder {
            bubblewrap_args: vec![],
            process: AsyncProcessWithCustomPipes::new(
                c"/usr/bin/bwrap",
                [c"/usr/bin/bwrap", c"--args", c"3"]
                    .into_iter()
                    .chain(command.iter().map(|k| k.as_c_str()))
                    .collect::<Vec<_>>(),
                &[] as &[&CStr],
            )
            .add_output(STDOUT_FILENO)
            .add_output(STDERR_FILENO),
            pipe_number: 4,
            command,
        }
        .add_args([
            c"--die-with-parent",
            //
            c"--chdir",
            c"/",
            c"--ro-bind",
            c"/lib64",
            c"/lib64",
            c"--ro-bind",
            c"/lib",
            c"/lib",
            c"--ro-bind",
            c"/usr/lib",
            c"/usr/lib",
            c"--tmpfs",
            c"/tmp",
            c"--tmpfs",
            c"/home/byte_heist",
            c"--setenv",
            c"HOME",
            c"/home/byte_heist",
            c"--ro-bind",
            lang_folder,
            c"/lang",
            c"--unshare-all",
            c"--new-session",
        ]);
        for (key, value) in lang.env {
            result = result.add_args([
                c"--setenv",
                &CString::new(*key).expect("Env keys cannot contain null bytes"),
                &CString::new(*value).expect("Env values can not contain null bytes"),
            ]);
        }

        for (external_folder, internal_folder) in lang.extra_mounts {
            result = result.add_args([
                c"--ro-bind",
                &CString::new(*external_folder).expect("Mount folders cannot contain null bytes"),
                &CString::new(*internal_folder).expect("Mount folders can not contain null bytes"),
            ]);
        }

        result
    }

    fn mount_string(mut self, path: &CStr, data: &'a [u8]) -> Self {
        let pipe_number = self.pipe_number;
        self.process = self.process.add_input(pipe_number, data);
        self.pipe_number += 1;
        self.add_args([c"--ro-bind-data"])
            .add_args([CString::new(format!("{}", pipe_number))
                .expect("Expected formating a number not to contain null bytes")])
            .add_args([path])
    }

    fn mount_folder(self, external_path: &CStr, internal_path: &CStr) -> Self {
        self.add_args([c"--bind", external_path, internal_path])
    }

    fn mount_ro_folder(self, external_path: &CStr, internal_path: &CStr) -> Self {
        self.add_args([c"--ro-bind", external_path, internal_path])
    }

    fn set_input(mut self, input: &'a [u8]) -> Self {
        self.process = self.process.add_input(STDIN_FILENO, input);
        self
    }

    async fn run(self) -> Result<RunCodeResult, RunProcessError> {
        let command = self.command;
        let Self {
            bubblewrap_args,
            process,
            ..
        } = self.add_args([c"--"]).add_args(command);

        let mut output = process
            .add_input(3, &bubblewrap_args)
            .output()?
            .await
            .map_err(RunProcessError::IOError)?;

        if let SignalOrStatus::Signal(signal) = output.result {
            if let Some(child_stderr) = output.outputs.get_mut(&STDERR_FILENO) {
                write!(
                    child_stderr,
                    "\nProcess was possibly killed by signal {}",
                    signal.as_str()
                )
                .expect("Formatting string should never fail")
            }
        }

        Ok(RunCodeResult {
            stdout: output
                .outputs
                .remove(&STDOUT_FILENO)
                .unwrap_or(String::new()),
            stderr: output
                .outputs
                .remove(&STDERR_FILENO)
                .unwrap_or(String::new()),
            exit_status: match output.result {
                // Seems signal + 128 is a standard way to report processes that where killed by a signal
                SignalOrStatus::Signal(e) => (e as i32) + 128,
                SignalOrStatus::Status(status) => status,
            },
        })
    }
}
