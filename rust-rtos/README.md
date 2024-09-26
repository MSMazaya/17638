# Rust Blinky

Name: Muhammad Sulthan Mazaya

Andrew ID: mmazaya

To run the program, make sure the current working directory is the root directory of this project:

```
$ tree -L 1
.
├── Cargo.lock
├── Cargo.toml
├── Embed.toml
├── gdb_commands.gdb
├── memory.x
├── README.md
├── src
└── target
```

To compile and run it, use the following command.

```
$ cargo run --release
```

It should automatically downloaded all the necessary crates. When finished, `arm-none-eabi-gdb` will be executed with commands on `gdb_commands.gdb` which automatically uses localhost:3333 as the remote target and load the binary release there. If necessary, it is possible to change the gdb command after compiling the source code. To do so, head to `.cargo/config.toml` and change the second line of the file with the new desired command.

```
[target.'cfg(all(target_arch = "arm", target_os = "none"))']
runner = "arm-none-eabi-gdb -x gdb_commands.gdb -se" # this will execute `arm-none-eabi-gdb -x gdb_commands.gdb -se [binary]`
                                                     # you can this to match your gdb path 
```
