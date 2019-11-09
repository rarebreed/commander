use futures_util::stream::{ StreamExt};
use std::process::{ Stdio };
use tokio::io::{ BufReader, AsyncBufReadExt };
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new("iostat");
    let command = cmd.args(vec!["2", "3"]).stdout(Stdio::piped());


    let mut child = command.spawn()
        .expect("failed to spawn command");

    let stdout = child.stdout().take()
        .expect("child did not have a handle to stdout");

    let mut reader = BufReader::new(stdout).lines();

    // Ensure the child process is spawned in the runtime so it can
    // make progress on its own while we await for any output.
    tokio::spawn(async {
        let status = child.await
            .expect("child process encountered an error");

        println!("child status was: {}", status);
    });

    while let Some(line) = reader.next().await {
        println!("{}", line?);
    }

    Ok(())
}