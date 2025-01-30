# Rimi

A fast CLI image manipulation tool, written for speed in rust.

## Features

### Features as of v0.1.6

- Batch operations
- Conversion
- Resizing
- Background removal
- Color changing

## Usage

### Batch operations

When an input of images above 1 is entered, the program will enter batch mode.
When we want to run a few operations on a large set of images we run:

```Shell
rimi convert -i ~/some-dir/*.jpg -o other-dir -f png
```

An example with resizing and conversion together:

```Shell
rimi resize -i * -o out-dir -w 3840 -H 2160 -f avif
```

You can also use name expressions to customise the names of the output images:

```Shell
rimi convert -i * -o dir/ -f avif -n this-image
```

Resulting in this:
```Shell
ls dir/

this_image_1.avif
this_image_2.avif
this_image_3.avif
...
```
#### A few notes about batch operations

- The number of operations done is parallel is roughly equal to the system core count
- A maximum of 10000 images can be manipulated at once

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
