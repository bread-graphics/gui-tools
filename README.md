# gui-tools

`gui-tools` is a toolkit containing functions and structures that may be use to build a GUI toolkit. A problem with several major GUI toolkits in the Rust ecosystem that I've noticed is the fact that they build their own tools to interact with native GUI libraries. This creates work for something that is probably reusable from another source. `gui-tools` aims to provide an unopinionated way to interface with low-level GUI libraries.

## WARNING

`gui-tools` is in an alpha state. The API will change on a whim. Feel free to experiment with this, but don't build anything that needs to be reliable on it.
