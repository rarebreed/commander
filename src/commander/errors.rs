use std::{error,
          error::Error,
          fmt,
          io};

// Define our error types. These may be customized for our error handling cases.
// Now we will be able to write our own errors, defer to an underlying error
// implementation, or do something in between.
#[derive(Debug, Clone)]
pub struct CommandError;

impl CommandError {
    pub fn new() -> Self {
        CommandError {}
    }
}

/// Generation of an error is completely separate from how it is displayed.
/// There's no need to be concerned about cluttering complex logic with the display style.
///
/// Note that we don't store any extra info about the errors. This means we can't state
/// which string failed to parse without modifying our types to carry that information.
impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error executing sub process")
    }
}

// This is important for other errors to wrap this one.
impl error::Error for CommandError {
    fn description(&self) -> &str {
        "error executing sub process"
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

impl From<CommandError> for io::Error {
    fn from(item: CommandError) -> io::Error {
        io::Error::new(io::ErrorKind::Other, item.description())
    }
}

impl From<io::Error> for CommandError {
    fn from(_: io::Error) -> CommandError {
        CommandError::new()
    }
}
