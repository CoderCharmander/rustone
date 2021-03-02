# Rustone
Rustone is a lightweight Minecraft server manager written in Rust. It aims to provide a both simple and
scalable way to manage Minecraft servers.

## Features

 - **Simple**: dive into creating a basic server on the CLI in just a few minutes
 - **Powerful**: use `rshttp`'s JSON API to manage your servers remotely
 - **Fast**: Rust is a native language, which makes Rustone really fast
 - **Cross platform**: Rust supports cross platform at its finest. Rustone does not stick to Windows nor to Linux.

## Installation
To install (compile from source), you need the [Rust toolchain](https://rust-lang.org).
The recommended way to install it is [Rustup](https://rustup.rs).

Now clone the Rustone repository with Git or download a ZIP or tar.gz file. Change
into its directory and type:
```bash
cargo install --path ./rscmd
```
This command will install `rscmd`, a command-line frontend to Rustone, good for
getting started.
You can also install `rshttp` to serve an API to start servers:
```bash
cargo install --path ./rshttp
```

## Getting started
After you have completed the installation of at least `rscmd`, you can get
started! After `USER_HOME/.cargo/bin` is in your PATH (on Windows, likely
yes), you can continue.

To create a new server, type `rscmd create <name> <version>`. Let's create
a 1.14.4 server named "test".
```bash
rscmd create test 1.14.4
```
You have to start it now:
```
rscmd start test
```
Rustone will now download the required files from PaperMC's website, and
start your server once done.

That's it! You can now connect to it from your local machine if you enter
`localhost` into the server address box.

Once you're done, type `stop` into the server console.