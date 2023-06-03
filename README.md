# rustutil

<img width="200" alt="logo" src="https://github.com/rustutil/.github/assets/62805599/81ddd70d-be1a-41bb-a0c1-902d22784832"/>

Rustutil is a utility package manager for rust applications.

## Runtime Dependencies

- Cargo

## Usage

```sh
rustutil --help
```

You can also specify no args to get help.  
When you first install a package, add `path/to/exe/../bin` to path. Then you may run a package by running its id in your shell.

## Experimental Features

You can enable experimental features when you build. Currently, the avalible features are

- **target-cache**  
  Enables the target cache, this saves the target directory to /targets.
  This saves rebuilding a packages depencencies when updating.
  The folder is deleted when the package is removed.
