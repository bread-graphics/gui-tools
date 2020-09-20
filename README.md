# gui-tools

[![Build Status](https://dev.azure.com/jtnunley01/gui-tools/_apis/build/status/not-a-seagull.gui-tools?branchName=master)](https://dev.azure.com/jtnunley01/gui-tools/_build/latest?definitionId=7&branchName=master)

`gui-tools` is a toolkit containing functions and structures that may be use to build a GUI toolkit. A problem with several major GUI toolkits in the Rust ecosystem that I've noticed is the fact that they build their own tools to interact with native GUI libraries. This creates work for something that is probably reusable from another source. `gui-tools` aims to provide an unopinionated way to interface with low-level GUI libraries.

## WARNING

`gui-tools` is in an alpha state. The API will change on a whim. Feel free to experiment with this, but don't build anything that needs to be reliable on it.

## Usage

See the `examples` directory for some example programs.

## Backends

* `xlib` - The X11 window system present on Linux and most non-OSX Unixes. At the moment, only Linux support is enabled.
* `win32` - The Windows API present on most Windows installations.

## Goals

* **Portability** - `gui-tools` aims to not only be usable on most computers, but provide consistent results on all of those computers. An app that looks one way on Windows should look the same on Linux.
* **Compatibility** - `gui-tools` is a `#![no_std]` crate (although both of the currently supported backends both require the C standard library). The idea is to reduce headaches intrinsic in porting `gui-tools` somewhere else.
* **Ease of Use** - The API of `gui-tools` should be intuitive enough to not have to constantly refer back to the docs.
* **Unopinionated** - `gui-tools` is just a set of tools for building GUIs. The toolkits that use `gui-tools` should not be constrained by any of `gui-tools`'s design decisions.
* **Fast** - `gui-tools` should preform well enough to the point where it isn't the slowest part of a non-trivial application.

## Things to Do

In order of priority, from highest to lowest:

* **Unpin from Nightly** - At the moment, `gui-tools` is pinned to the Nightly version of the Rust compiler. Once `min_const_generics` and `const_fn` are stabilized, this should be relatively easy.
* **Document API** - A lot of items in the API don't have good documentation of how they work.
* **`image-rs` Support** - I'd like to add some kind of `From<image_rs::RgbImage>` for the `gui_tools::image::Image` struct, to make image loading easier.
* **Text Rendering** - `gui-tools` can already render images. I'd think that if we use [`fontdue`](https://crates.io/crates/fontdue) to render vector images this should be a relatively simple process.
* **Appkit Backend** - [AppKit](https://developer.apple.com/documentation/appkit) is the OSX backend to the GUI toolkit. The [`objc`](https://crates.io/crates/objc) crate is probably the best way to interface with this. [XQuartz](https://www.xquartz.org/) exists but we shouldn't rely on its existence.
* **DOM Backend** - Given the progress that's being made with WebAssembly, there should be a way to directly interface with the DOM at some point in the future. Barring that, we can just call the JS functions with something like [`wasm-bindgen`](https://crates.io/crates/wasm-bindgen).
* **Async Support** - I want `gui-tools` to have support for the asynchronous ecosystem. This would be feature-gated, of course; I don't want `gui-tools` to be 100% async.
* **BSD** - The X11 backend will probably work for all of the versions of BSD out there. We just need to make sure that it does; most CI services don't really have a BSD option.
* **Reduce Unsafe** - There's a lot more unsafe code in this crate than there probably needs to be.

## License

`gui-tools` is dual-licensed under the MIT License and the Apache 2.0 License. For more information, see the `LICENSE-MIT` and `LICENSE-Apache` files in the root of the repository.
