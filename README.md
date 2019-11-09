# commander

This is a simple library that makes it a little easier to create a subprocess from rust

## TODO

This is bleeding edge stuff. and async just got working.  The  main function is hard coded now,  but the 
rest of lib.rs can use the async code now, instead of using multithreading.  Or perhaps we can include
both now.

Currently, tokio is built locally, since tokio 0.2.0-alpha.6 still doesn't have what we need