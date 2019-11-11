use futures_util::stream::{ StreamExt};
use std::process::{ Stdio };
use tokio::io::{ BufReader, AsyncBufReadExt };
use tokio::process::Command;
use log::{ info, error };

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    // 0th arg will be 'commander'
    if args.len() < 2 {
        error!("Must supply at least 1 arg");
        return Ok(())
    }

    // 1st arg is our main command
    let mut cmd = Command::new(args.get(1).unwrap());
    // any remaining values are args to the child command
    let command = cmd.args(&args[2..]).stdout(Stdio::piped());

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

        info!("child status was: {}", status);
    });

    while let Some(line) = reader.next().await {
        println!("{}", line?);
    }

    Ok(())
}