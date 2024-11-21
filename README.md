# Rimi

A simple image manipulation program for the terminal
written in rust.

Features coming as often as I can make them!

## Usage

```text
Simple in-development image manipulation tool

Usage: rimi [OPTIONS] <FILENAME> [COMMAND]

Commands:
  convert  Convert an image
  resize   Resize an image
  help     Print this message or the help of the given subcommand(s)

Arguments:
  <FILENAME>  Input image filename

Options:
  -o, --output <OUTPUT>  Output image
  -h, --help             Print help
  -V, --version          Print version```

```bash
# Specift a source image and it's output format
rimi FILENAME convert --format png

# Resizing images
rimi FILENAME.png resize -w 1200 -h 800
```
