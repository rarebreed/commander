//! This module will handle the abstraction of running commands, either locally or remotely on a machine.
pub mod commander;

use commander::{ 
  errors::{ CommandError },
  common::{ RunOpts, run },
  child_ext::{ Communicate }
};

use std::{
  { thread },
  thread::{ JoinHandle },
  io::{ Result as IoResult },
  process::{ Child, Command, ExitStatus },
  time::{ Duration, Instant },
};
use log::{info, error};
use tokio::process::{ Command as CommandAsync, Child as ChildAsync, ChildStdout };
use tokio::io::{ BufReader };


/// Trait for a type that can execute a command either synchronously or async
pub trait Executor {
  type ExecResult;

  fn run(&mut self, opts: RunOpts) -> Result<Self::ExecResult, CommandError>;
}


pub struct CommandResult {
  pub status: Option<ExitStatus>,
  pub output: String,  // FIXME: If this is a long running process, this can consume memory
  pub child: Option<Child>
}

pub struct AsyncResult {
  pub status: Option<ExitStatus>,
  pub output: BufReader<ChildStdout>,
  pub child: Option<ChildAsync>
}

impl CommandResult {
  /// Wrapper to call into Child's send()
  pub fn send(subproc: &mut Option<Child>, content: String) -> IoResult<()> {
    let child = match subproc.take() {
      Some(mut child) => {
        child.send(content)?;
        Some(child)
      },
      None => {
        error!("There is no child subprocess to send to");
        None
      }
    };
    *subproc = child;  // Whenever you take() an Option, put the original value back in case it was a Some(x)
    Ok(())
  }

  /// Waits for the child process to exit
  /// 
  /// Useful in conjunction with Executor.run_thread().  This function will block until the process ends, or the timeout
  /// (in milliseconds expires)
  pub fn wait(&mut self, timeout: u64) -> Option<ExitStatus> {
    let start_time = Instant::now();
    let duration = Duration::from_millis(timeout);
    let mut exit_status: Option<ExitStatus> = None;

    while start_time.elapsed() < duration {
      if let Some(mut child) = self.child.take() {
        match child.try_wait() {
          Ok(Some(status)) => {
            self.status = Some(status);
            exit_status = Some(status);
            break;
          },
          Err(e) => {
            error!("Child process encountered error: {}", e);
            break;
          },
          _ => {
            
          }
        }
        self.child = Some(child);
      }
      thread::sleep(Duration::from_millis(200));
    };
    exit_status
  }
}

impl Executor for Command {
  type ExecResult = CommandResult;

  /// Executes a subprocess and waits for it to complete
  fn run(&mut self, opts: RunOpts) -> Result<CommandResult, CommandError> {
    let thrd_handle = run_thread(self, opts)?;    
    
    match thrd_handle.join() {
      Ok(result) => {
        match result.status {
          None => {
            info!("No exit code for the child");
          },
          Some(_stat) => {
            error!("Exit status is {}", _stat);
          }
        }
        Ok(result)
      },
      Err(_) => {
        error!("Could not run process");
        Err(CommandError::new())
      }
    }
  }
}

impl Executor for CommandAsync {
  type ExecResult = AsyncResult;

  fn run(&mut self, _opts: RunOpts) -> Result<AsyncResult, CommandError> {
    let mut child = self.spawn()
        .expect("failed to spawn command");

    let stdout = child.stdout().take()
        .expect("child did not have a handle to stdout");

    let reader = BufReader::new(stdout);
    
    Ok(AsyncResult {
      status: None,
      output: reader,
      child: Some(child)
    })
  }
}

/// Spawns the subprocess, reads the stdout in a separate thread, and returns the thread handle.
/// 
/// The thread itself returns a CommandResult.  If the subprocess was unsuccessful, a CommandResult
/// is still returned but with no status
pub fn run_thread(cmd: &mut Command, opts: RunOpts) -> IoResult<JoinHandle<CommandResult>> {
  let mut process = cmd.spawn()?;

  // FIXME: When async stabilizes, use task instead of thread
  let thrd_handle = thread::spawn(move || {
    let (status, output) = run(&mut process, opts);
        CommandResult {
          output,
          status,
          child: Some(process)
        }
    }
  );
  Ok(thrd_handle)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::process::Stdio;
  use std::thread;
  use tokio::runtime::Runtime;
  use futures_util::stream::StreamExt;
  use tokio::io::AsyncBufReadExt;

  #[test]
  fn test_simple() {
    let mut cmd = Command::new("iostat");
    let ref mut command = cmd.args(vec!["2", "3"])
      .stdout(Stdio::piped())
      .stdin(Stdio::piped());
    
    let subproc = command.run(RunOpts::default());

    match subproc {
      Ok(result) => {
        println!("Output is {}", result.output);
      },
      _ => ()
    }
  }

  /// This tests a manual implementation of creating a separate thread to do some kind of task.
  #[test]
  fn test_multithread() {
    let subproc = Command::new("iostat")
      .args(vec!["2", "3"])
      .stdout(Stdio::piped())
      .stdin(Stdio::piped())
      .spawn();

    let thrd_handle = thread::spawn(move || {
      // This will be our accumulated string output
      let mut saved_output = String::new();
      let mut exit_code: Option<ExitStatus> = None;

      // subproc might have returned an io::Error, so match for that
      match subproc {
        Ok(mut process) => {
          let (exit, output) = run(&mut process, RunOpts::default());
          saved_output = output;
          exit_code = exit;
        }
        Err(_) => {
          info!("Could not launch subprocess");
          assert!(false);
        }
      }

      // Return a tuple of the exit code and the saved output
      (exit_code, saved_output)
    });

    // Join with the new thread.  It will finish when try_wait() no longer returns Ok(None)
    match thrd_handle.join() {
      Ok((exit_code, output)) => {
        assert!(output.len() > 0);
        match exit_code {
          Some(exit) => assert!(exit.success()),
          None => assert!(false),
        }
      }
      Err(_) => {
        error!("Could not run process");
        assert!(false);
      }
    }
  }

  #[test]
  /// Tests getting the console
  fn test_keyboard() {
    println!("Please enter something and hit Enter:\n");
    let keyboard = std::io::stdin();
    let mut buffer = String::new();
    match keyboard.read_line(&mut buffer) {
      Ok(n) => {
        info!("bytes read = {}", n);
        info!("You entered: {}", buffer);
        assert!(true);
      }
      Err(e) => {
        error!("Error {}", e);
        assert!(false);
      }
    }
  }

  // #[test]  This test doesn't play nice with other tests
  fn _test_ssh() {
    let mut child = Command::new("ssh")
      .args(vec!["stoner@localhost"])
      .spawn()
      .unwrap();

    loop {
      match child.try_wait() {
        Ok(Some(status)) => {
          println!("exited with: {}", status);
          break;
        },
        Ok(None) => {
            println!("status not ready yet, let's really wait");
            let res = child.wait();
            println!("result: {:?}", res);
        }
        Err(e) => {
          println!("error attempting to wait: {}", e);
          break;
        }
      }
    }
  }

  #[test]
  fn test_async() -> Result<(), Box<dyn std::error::Error>> {
    // we will use tokio runtime here, since we canr use the tokio::main macro
    let mut rt = Runtime::new()?;

    rt.block_on(async {
      let mut cmd = CommandAsync::new("iostat");

      let _cmd = cmd
        .args(vec!["2", "3"])
        .stdout(Stdio::piped());

      let opts = RunOpts::default();
      let stat = cmd.run(opts);
      match stat {
        Ok(result) => {
          if let Some(child) = result.child {
            tokio::spawn(async {
              let status = child.await.expect("No child process");
              assert!(status.success());
            });

            let mut lines = result.output.lines();
            while let Some(Ok(line)) = lines.next().await {
              println!("{}", line);
            }
          }
        },
        Err(e) => {
          eprintln!("Error: {}", e);
        }
      };
      
    });
    
    Ok(())
  }
}