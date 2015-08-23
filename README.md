A Rust wrapper for libevdev

Why a libevdev wrapper?
-----------------------
The evdev protocol is simple, but quirky, with a couple of behaviors that
are non-obvious. libevdev transparently handles some of those quirks.

The evdev crate on [1] is an implementation of evdev in Rust. Nothing wrong
with that, but it will miss out on any more complex handling that libevdev
provides.

[1] https://github.com/cmr/evdev/blob/master/src/lib.rs
