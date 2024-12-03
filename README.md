# Rimi

A simple image manipulation program for the terminal written in rust.

## Features

### Features as of v0.1.1

- Conversion
- Resizing

### To-do

- [ ] Image format decoding

## Usage

```text
Simple in-development image manipulation tool

Usage: rimi [OPTIONS] <FILENAME> [COMMAND]

Commands:
  convert, -c  Convert an image
  resize, -r   Resize an image
  help         Print this message or the help of the given subcommand(s)

Arguments:
  <FILENAME>  Input image filename

Options:
  -o, --output <OUTPUT>  Output image
  -x, --overwrite        Overwrite any existing files when saving the image
  -h, --help             Print help
  -V, --version          Print version
```

### Image conversion

#### Auto-detected format

```bash
rimi image.jpg -o new.png -c
```

#### Format as argument

This will create a new file called image.jpg

```bash
rimi image.png -c -f jpg
```

### Resizing

Resize images like so:

```bash
rimi img.png -r -x 1920 -y 1080
```

You can also specify image sampling filters. Documented here:
[Image Filter Type](https://docs.rs/image/0.25.5/image/imageops/enum.FilterType.html).

```bash
# Supports image formats when resizing
rimi image.png resize -x 1920 -y 1080 -t lanczos
```

## Credits

Thanks to the creators of the image crate on crates<!---->.io:
<https://crates.io/crates/image>.
