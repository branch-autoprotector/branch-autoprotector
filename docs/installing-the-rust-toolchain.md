# Installing the Rust toolchain

The Branch Autoprotector is written in Rust, so you’ll need a Rust environment to build and deploy this service.
However, `rustup` makes it fairly straightforward to get the Rust toolchain up and running.

1. Follow the [instructions on rustup.rs][rustup.rs] to install the stable Rust toolchain through `rustup`.
   Select the default installation to get the stable compiler.

2. Continue with a new shell.
   Alternatively, configure your current shell to recognize Rust commands:

   ```shell
   $ source $HOME/.cargo/env
   ```

3. Install `cargo deb` to be able to generate Debian packages if you’d like to deploy this service later on:

   ```shell
   $ cargo install cargo-deb
   ```

[rustup.rs]: https://rustup.rs/