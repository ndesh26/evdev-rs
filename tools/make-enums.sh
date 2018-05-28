#!/usr/bin/env bash

set -eux

./tools/make-event-names.py evdev-sys/libevdev/include/linux/input-event-codes.h evdev-sys/libevdev/include/linux/input.hpython tools/make-event-names.py evdev-sys/libevdev/include/linux/input-event-codes.h evdev-sys/libevdev/include/linux/input.h > src/enums.rs
