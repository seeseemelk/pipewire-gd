# pipewire-gd

Pipewire interface library, specifically designed for capturing and rendering streams within a Godot context.

Pipewire is a *nix specific library.  As such, this project is designed and maintained such that building is expected to be done on a Linux system.

Primary use-cases for this library include
- Applying advanced visual effects to captured application that are not possible in software like OBS
- Mirroring desktop windows into VR/XR contexts

### Getting started:
0. Make sure you have libpipewire-0.3 headers available on your build system.
    - ex. on a Pop_OS system you can do this easily with<br>
      `apt install libpipewire-0.3-dev`
1. Clone this repository with submodules.
    - `git clone --recurse-submodules https://github.com/erodozer/pipewire-gd.git`
    - `cd pipewire-gd`
2. Update to the latest `godot-cpp`.
    - `git submodule update --remote`
2. Build a debug binary for the current platform.
    - `scons`
3. Import, edit, and play `project/` using Godot Engine 4+.
    - `godot --path project/`

### Repository structure:
- `project/` - Godot project boilerplate.
  - `addons/example/` - Files to be distributed to other projects.¹
  - `demo/` - Scenes and scripts for internal testing. Not strictly necessary.
- `src/` - Source code of this extension.
- `godot-cpp/` - Submodule needed for GDExtension compilation.

¹ Before distributing as an addon, all binaries for all platforms must be built and copied to the `bin/` directory. This is done automatically by GitHub Actions.
