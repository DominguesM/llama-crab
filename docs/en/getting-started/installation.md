# Installation

`llama-crab` is distributed on [crates.io](https://crates.io/crates/llama-crab)
and is also buildable from a Git checkout. The default features compile
the CPU (OpenMP) and — on Apple Silicon — Metal backends, so most users
can add the dependency and start building.

## 1. Add the dependency

=== "Stable (crates.io)"

    ```toml title="Cargo.toml"
    [dependencies]
    llama-crab = "0.1"
    ```

=== "Git main branch"

    ```toml title="Cargo.toml"
    [dependencies]
    llama-crab = { git = "https://github.com/DominguesM/llama-crab", branch = "main" }
    ```

=== "Local checkout"

    ```toml title="Cargo.toml"
    [dependencies]
    llama-crab = { path = "../llama-crab" }
    ```

!!! tip "Pin the llama.cpp version"

    The crate pins `llama.cpp` to a specific commit, so two builds of
    the same `llama-crab` version always produce the same native
    library. You can see the pinned commit on the README badge or
    through `cargo tree -p llama-crab-sys`.

## 2. Pick a backend

The default features give you a working binary on the most common
platforms, but you almost always want to be explicit:

=== "Apple Silicon (macOS)"

    ```toml title="Cargo.toml"
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["metal", "openmp"] }
    ```

=== "Linux + NVIDIA GPU"

    ```toml title="Cargo.toml"
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["cuda", "openmp"] }
    ```

=== "Linux + AMD GPU"

    ```toml title="Cargo.toml"
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["rocm", "openmp"] }
    ```

=== "Vulkan (any vendor)"

    ```toml title="Cargo.toml"
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["vulkan", "openmp"] }
    ```

=== "CPU only"

    ```toml title="Cargo.toml"
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["openmp"] }
    ```

=== "Android / mobile"

    See the dedicated [Mobile distribution](../guides/mobile.md) guide.

See the [Cargo features reference](../reference/cargo-features.md) for
the complete list of features and what each one toggles.

## 3. System requirements

The build script compiles `llama.cpp` from source. Make sure the
following are available **before** running `cargo build`:

=== "macOS"

    ```bash
    # Xcode Command Line Tools
    xcode-select --install

    # CMake (Homebrew, or use the one from CLT if present)
    brew install cmake
    ```

=== "Debian / Ubuntu"

    ```bash
    sudo apt update
    sudo apt install -y build-essential cmake
    ```

=== "Fedora / RHEL"

    ```bash
    sudo dnf install -y gcc gcc-c++ cmake make
    ```

=== "Windows (MSVC)"

    ```powershell
    # Install Visual Studio 2022 with the "Desktop development with C++"
    # workload, then:
    winget install Kitware.CMake
    ```

!!! warning "First build is slow"

    Compiling every llama.cpp backend takes ~3 minutes on a 16-core
    machine the first time. Subsequent builds are cached. To cut the
    cold build, disable the backends you don't need — see step 2.

## 4. Verify the toolchain

After the install, run a quick `cargo build` to make sure CMake, the
compiler and the C++ standard library are all reachable:

```bash
cargo new hello-crab --bin
cd hello-crab
# Add the dependency shown in step 1, then:
cargo build --release
```

A successful build prints something like:

```
   Compiling llama-crab-sys v0.1.300 (...)
   Compiling llama-crab v0.1.300 (...)
    Finished `release` profile [optimized] [..]
```

You're ready to write your [first program](first-program.md).

## Optional: download a model

The rest of the guide assumes you have a GGUF file on disk. The
easiest way to grab a known-good one is the helper script:

=== "Smallest text model (Qwen2.5 0.5B)"

    ```bash
    ./scripts/download_models.sh smol
    # → models/qwen2.5-0.5b-instruct-q4_k_m.gguf
    ```

=== "Embedding model (BGE-small)"

    ```bash
    ./scripts/download_models.sh bge
    # → models/bge-small-en-v1.5-q4_k_m.gguf
    ```

=== "Vision model (Gemma 4 + mmproj)"

    ```bash
    ./scripts/download_models.sh gemma4
    # → models/gemma-4-E4B-it-Q4_K_M.gguf
    # → models/mmproj-gemma-4-E4B-it-BF16.gguf
    ```

See [`scripts/download_models.sh`](https://github.com/DominguesM/llama-crab/blob/main/scripts/download_models.sh)
for the full list of supported targets.

## Next steps

- Walk through [Your first program](first-program.md) — a 50-line
  `main.rs` that exercises the most common paths.
- Skim the [Cargo features reference](../reference/cargo-features.md)
  to know what's enabled by default and what to toggle for your
  target.
- Jump straight to a [feature guide](../features/index.md) that
  matches what you want to build.
