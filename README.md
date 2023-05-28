# rustutil

Rustutil is a utility package manager for rust applications.

## Dependencies

- Cargo

## Building

```sh
cargo build
```

## Usage

If you built by yourself

```sh
cargo run -q -- [args]
```

If you downloaded an executable

```sh
path/to/executable [args]
```

Specify no args to get help.  
When you first install a package, go to where you placed the executable and add the `bin` folder to path. Then you may run a package by typing in its ID.

## Examples

To install a test package:

```sh
rustutil add helloworld
```

To run the test package

```sh
helloworld
```
