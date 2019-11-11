# commander

This is a simple library that makes it a little easier to create a subprocess from rust

## TODO

This is bleeding edge stuff and async just got working.  

Currently, tokio is built locally, since tokio 0.2.0-alpha.6 still doesn't have what we need.

One of the remaining things that needs to be done is add the ability to send commands to a child process.
This is done via the Communicate trait for the synchronous code.  Something similiar still needs to be
done for the async code.  The most common use case for this is when you need to run a subprocess as sudo.

When you run a child process as sudo, the real main command is `sudo`.  It will prompt for a password on
a pseudo tty.  However, this can be changed by passing in extra args to sudo (-s -k).

## Limitations with async

As mentioned above.  This is still bleeding edge.  tokio hasn't updated their alpha crate since Sep 2019.
Also, rustc doesn't currently support async fn's in traits.  And even if it did, you have to design async
traits only for async code.  In other words, you can't and shouldn't have traits where the fn impls can
work either synchronously or asynchronously.  This is probably a good thing.  It's better to be explicit
and essentially have sync/async be codified in the type itself.  Although you can use sync functions in
async code, the inverse is not true.  Even then, you must be careful that sync code in an async fn or
block doesn't block for too long.

## Running it as an executable

Although commander is designed as a library primarily, it can be used as an executable.

```bash
cargo run ls -al
cargo run iostat 2 3
```

Or you can build and run manually

```bash
cargo build --release
target/release/commander ls -alh
```