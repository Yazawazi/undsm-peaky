# Peaky - UnDSM

*Very poor encryption, but I like it*

## Description

- Unpack `.dsm` file
- Pack `.dsm` file

## Build

```bash
cargo build --release
```

## Usage

```text
Usage: undsm [OPTIONS] --input <INPUT> <--unpack|--pack>

Options:
  -u, --unpack           Unpack a .dsm file
  -p, --pack             Pack a file to .dsm
  -f, --force            Force writing to output file
  -i, --input <INPUT>    Input file
  -o, --output <OUTPUT>  Output file
  -h, --help             Print help information
  -V, --version          Print version information
```
