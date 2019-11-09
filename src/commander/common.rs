use std::{
  process::{ Child, ExitStatus },
  io::{ Result as IoResult, Read }
};
use log::{error, info};
use super::child_ext::{ Communicate };


#[derive(Clone)]
pub struct RunOpts {
  pub pw: Option<String>,
  pub showout: bool
}

impl RunOpts {
  pub fn new(pw: Option<String>, showout: bool) -> Self {
    RunOpts {
      pw, showout
    }
  }
}

impl Default for RunOpts {
  fn default() -> Self {
    RunOpts {
      pw: None,
      showout: true
    }
  }
}

pub fn send_pw(process: &mut Child, input: Option<String>) -> IoResult<()> {
  match input {
    None => Ok(()),
    Some(pass) => {
      process.send(pass)
    }
  }
}

/// Get output from the ChildStdout|ChildStderr
/// TODO: look into unstable, and use async here
pub fn read_from<T: Read>(output: &mut T, showout: bool) -> Option<String> {
  let mut buffer: [u8; 512] = [0; 512]; // Buffer for reading from stdout
  let mut out = String::new();          // Buffer for the saved output

  // Read from the ChildStdout until it has no data
  // FIXME: What if ChildStdout is constantly streaming?  Will this consume too much memory?
  // TODO: If we use async, return Stream<String> instead
  while let Ok(size) = output.read(&mut buffer) {
    if size != 0 {
      // Convert what we have so far in the buffer into a String
      // FIXME:  What if we don't have (valid) utf8 in the buffer?
      if let Ok(body) = String::from_utf8(buffer[0..size].to_vec()) {
        if showout {
          print!("{}", body);
        }
        out.push_str(&body);
      }
    } else {
      break;
    }
  }
  Some(out)
}

/// Given a child process, will run to completion.  Unlike wait_output, this function will get the stdout while the
/// process is still running.
///
/// It is ideal to run this in a separate thread, otherwise the loop will eat up the thread it is running on
/// TODO: Right now, we have no way to do anything with the output while it is being recieved.  See if it is possible
/// to make this a futures::future::Stream.  Alternatively, and perhaps more easily, can add a callback with a signature
/// of Fn(String) -> ()
pub fn run(process: &mut Child, opts: RunOpts) -> (Option<ExitStatus>, String) {
  let mut saved_output = String::new();
  let mut exit_code: Option<ExitStatus> = None;

  // Check if we have a password.  If so, look at the stderr, and wait for a prompt
  send_pw(process, opts.pw).expect("Could not pass input to child");

  // The stdout may have closed at any time, so check during our loop.
  loop {
    match process.try_wait() {
      Ok(None) => {
        // The take() is required, if it is not used, rustc will complain that process.stdout was moved out.
        if let Some(mut out) = process.stdout.take() {
          // Read from the buffer until the stdout has no more available data
          if let Some(data) = read_from(&mut out, opts.showout) {
            saved_output.push_str(&data);
          }

          // take() replaces value with None, so we need to put it back in
          process.stdout = Some(out);
        }
      }
      Ok(Some(status)) => {
        info!("Process exited with status {}", status);
        if let Some(mut out) = process.stdout.take() {
          if let Some(data) = read_from(&mut out, opts.showout) {
            saved_output.push_str(&data);
          }
        }
        exit_code = Some(status);
        break;
      }
      Err(e) => {
        error!("Error with process: {}", e);
        break;
      }
    }
    std::thread::sleep(std::time::Duration::from_millis(100));
  }

  (exit_code, saved_output)
}