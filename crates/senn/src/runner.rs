//! Running external programs.
//!
//! Every tool Senn orchestrates (nix, tofu, nixos-anywhere, deploy-rs, ssh, the secrets
//! backend) runs through a [`Runner`], so tests can replace it with a fake.

use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::error::{Error, Result};

/// A program invocation, built with chained methods.
///
/// `env` and `stdin` can carry secrets. [`Debug`] prints only the names of environment
/// variables and hides stdin, so a `Cmd` is safe to log.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Cmd {
    /// The program, a name on `PATH` or a path.
    pub program: String,
    /// The arguments, in order.
    pub args: Vec<String>,
    /// Extra environment variables, added to the inherited environment.
    pub env: Vec<(String, String)>,
    /// Working directory; the current one if `None`.
    pub cwd: Option<PathBuf>,
    /// Text written to stdin; stdin is inherited if `None`.
    pub stdin: Option<String>,
    /// Capture stdout instead of passing it through to the terminal.
    pub capture: bool,
}

impl Cmd {
    /// Starts an invocation of `program` without arguments.
    #[must_use]
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            ..Self::default()
        }
    }

    /// Appends one argument.
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Appends several arguments.
    #[must_use]
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// Sets an environment variable for the program.
    #[must_use]
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    /// Sets the working directory.
    #[must_use]
    pub fn cwd(mut self, dir: impl Into<PathBuf>) -> Self {
        self.cwd = Some(dir.into());
        self
    }

    /// Writes `input` to stdin, then closes it.
    #[must_use]
    pub fn stdin(mut self, input: impl Into<String>) -> Self {
        self.stdin = Some(input.into());
        self
    }

    /// Captures stdout instead of passing it through to the terminal.
    #[must_use]
    pub const fn capture(mut self) -> Self {
        self.capture = true;
        self
    }
}

impl fmt::Debug for Cmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let env: Vec<&str> = self.env.iter().map(|(k, _)| k.as_str()).collect();
        f.debug_struct("Cmd")
            .field("program", &self.program)
            .field("args", &self.args)
            .field("env", &env)
            .field("cwd", &self.cwd)
            .field("stdin", &self.stdin.as_ref().map(|_| "<hidden>"))
            .field("capture", &self.capture)
            .finish()
    }
}

/// The result of a finished program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    /// Exit code, `None` if a signal ended the program.
    pub code: Option<i32>,
    /// Captured stdout, empty unless the command was built with [`Cmd::capture`].
    pub stdout: String,
}

/// Runs programs. [`System`] runs real processes; tests use a fake.
pub trait Runner {
    /// Runs `cmd` to completion. A non-zero exit is not an error here; see [`run`].
    ///
    /// # Errors
    ///
    /// [`Error::Spawn`] if the program cannot start, [`Error::Io`] if stdin or stdout
    /// fails, [`Error::Protocol`] if captured stdout is not UTF-8.
    fn exec(&self, cmd: &Cmd) -> Result<Output>;
}

/// Runs `cmd` and returns its stdout (empty unless captured).
///
/// # Errors
///
/// The errors of [`Runner::exec`], and [`Error::Failed`] if the program exits non-zero.
pub fn run(runner: &dyn Runner, cmd: &Cmd) -> Result<String> {
    let out = runner.exec(cmd)?;
    if out.code == Some(0) {
        Ok(out.stdout)
    } else {
        Err(Error::Failed {
            program: cmd.program.clone(),
            code: out.code,
        })
    }
}

/// Runs real processes. Stderr always goes to the terminal.
#[derive(Debug, Clone, Copy, Default)]
pub struct System;

impl Runner for System {
    fn exec(&self, cmd: &Cmd) -> Result<Output> {
        let mut command = Command::new(&cmd.program);
        command
            .args(&cmd.args)
            .envs(cmd.env.iter().map(|(k, v)| (k, v)));
        if let Some(dir) = &cmd.cwd {
            command.current_dir(dir);
        }
        command.stdin(if cmd.stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::inherit()
        });
        command.stdout(if cmd.capture { Stdio::piped() } else { Stdio::inherit() });
        command.stderr(Stdio::inherit());

        let mut child = command.spawn().map_err(|source| Error::Spawn {
            program: cmd.program.clone(),
            source,
        })?;
        // ponytail: stdin is written in full before stdout is read; fine for small inputs
        // (requests, secrets), a writer thread if a large input ever meets a chatty program.
        if let (Some(input), Some(mut pipe)) = (&cmd.stdin, child.stdin.take()) {
            pipe.write_all(input.as_bytes())?;
        } // The pipe is dropped here, so the program sees end of input.
        let out = child.wait_with_output()?;
        let stdout = String::from_utf8(out.stdout)
            .map_err(|_| Error::Protocol(format!("`{}` wrote output that is not UTF-8", cmd.program)))?;

        Ok(Output {
            code: out.status.code(),
            stdout,
        })
    }
}

/// Test double for [`Runner`]: records every command and answers with a closure.
#[cfg(test)]
pub mod fake {
    use std::cell::RefCell;

    use super::{Cmd, Output, Runner};
    use crate::error::Result;

    type Respond = Box<dyn Fn(&Cmd) -> Option<String>>;

    /// A [`Runner`] that runs nothing.
    pub struct Fake {
        calls: RefCell<Vec<Cmd>>,
        respond: Respond,
    }

    impl Fake {
        /// `respond` returns `Some(stdout)` for exit code 0, `None` for exit code 1.
        pub fn new(respond: impl Fn(&Cmd) -> Option<String> + 'static) -> Self {
            Self {
                calls: RefCell::new(Vec::new()),
                respond: Box::new(respond),
            }
        }

        /// All commands so far, in order.
        pub fn calls(&self) -> Vec<Cmd> {
            self.calls.borrow().clone()
        }

        /// The programs of all commands so far, in order.
        pub fn programs(&self) -> Vec<String> {
            self.calls
                .borrow()
                .iter()
                .map(|c| c.program.clone())
                .collect()
        }
    }

    impl Runner for Fake {
        fn exec(&self, cmd: &Cmd) -> Result<Output> {
            self.calls.borrow_mut().push(cmd.clone());
            Ok((self.respond)(cmd).map_or_else(
                || Output {
                    code: Some(1),
                    stdout: String::new(),
                },
                |stdout| Output { code: Some(0), stdout },
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A shell command on the test platform: `sh -c` on Unix, `cmd /C` on Windows.
    fn shell(script: &str) -> Cmd {
        if cfg!(windows) {
            Cmd::new("cmd").args(["/C", script])
        } else {
            Cmd::new("sh").args(["-c", script])
        }
    }

    #[test]
    fn run_captures_stdout() {
        let out = run(&System, &shell("echo hello").capture()).unwrap();
        assert_eq!(out.trim_end(), "hello");
    }

    #[test]
    fn run_feeds_stdin() {
        // `sort` with one line prints it unchanged; Windows has no `cat`.
        let cmd = Cmd::new(if cfg!(windows) { "sort" } else { "cat" });
        let out = run(&System, &cmd.stdin("piped\n").capture()).unwrap();
        assert_eq!(out.trim_end(), "piped");
    }

    #[test]
    fn run_fails_on_nonzero_exit() {
        let err = run(&System, &shell("exit 1")).unwrap_err();
        assert!(matches!(err, Error::Failed { code: Some(1), .. }), "{err:?}");
    }

    #[test]
    fn exec_reports_exit_code() {
        let out = System.exec(&shell("exit 3")).unwrap();
        assert_eq!(out.code, Some(3));
    }

    #[test]
    fn missing_program_is_a_spawn_error() {
        let err = run(&System, &Cmd::new("senn-test-no-such-program")).unwrap_err();
        assert!(matches!(err, Error::Spawn { .. }), "{err:?}");
    }

    #[test]
    fn debug_hides_secrets() {
        let cmd = Cmd::new("tofu")
            .env("TF_VAR_token", "s3cret")
            .stdin("s3cret");
        let debug = format!("{cmd:?}");
        assert!(debug.contains("TF_VAR_token"), "{debug}");
        assert!(!debug.contains("s3cret"), "{debug}");
    }
}
