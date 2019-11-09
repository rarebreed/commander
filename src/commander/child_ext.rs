use std::{
  io::{ Result as IoResult, Write, ErrorKind },
  process::{ Child }
};
use log::{error, info};

pub trait Communicate {
  fn send(&mut self, input: String) -> IoResult<()>;
}

impl Communicate for Child {
  fn send(&mut self, mut input: String) -> IoResult<()> {
    match self.try_wait() {
      Ok(None) => {
        let mut stdin = self.stdin.take().expect("No stdin to pass input to");
        input.push_str("\n");
        let res = stdin.write_all(input.as_bytes());  // send to child's stdin

        // Since we took stdin out, we have to put it back, otherwise if we try to take() it again later
        // it will be None even if it had a Some value
        self.stdin = Some(stdin);
        res
      },
      Ok(Some(status)) => {
        info!("Process exited with status {}", status);
        Err(std::io::Error::new(ErrorKind::Other, "Process already exited!"))
      },
      Err(e) => {
        error!("Error with process: {}", e);
        Err(e)
      }
    }
  }
}