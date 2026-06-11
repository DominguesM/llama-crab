//! `build.rs` for `llama-crab-sys`.
//!
//! Responsibilities (in order):
//! 1. Detect host platform, target architecture and supported backends.
//! 2. Configure CMake build of `llama.cpp` (`LLAMA_BUILD_TESTS=OFF`, etc.).
//! 3. Compile additional C++ wrappers (chat templates, JSON schema, multimodal).
//! 4. Run `bindgen` over `wrapper.h` to generate Rust FFI bindings.
//! 5. Emit `cargo:` directives to wire up include dirs, lib paths and link flags.
//!
//! The file is intentionally split into many small helpers so each concern
//! lives in its own function — easier to read and to maintain than the
//! monolithic build.rs of older Rust bindings for llama.cpp.

#![allow(clippy::too_many_lines, clippy::print_stdout)]

use std::{env, path::PathBuf, process::Command};

use cmake::Config;
use glob::glob;
use walkdir::WalkDir;

// ---------------------------------------------------------------------------
// Platform detection
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum TargetOs {
    WindowsMsvc,
    WindowsGnu,
    Macos,
    Linux,
    Android,
    Other,
}

impl TargetOs {
    fn from_env() -> Self {
        let triple = env::var("TARGET").unwrap_or_default();
        if triple.contains("android") {
            Self::Android
        } else if triple.contains("darwin") || triple.contains("apple") {
            Self::Macos
        } else if triple.contains("windows-msvc") {
            Self::WindowsMsvc
        } else if triple.contains("windows-gnu") || triple.contains("windows") {
            Self::WindowsGnu
        } else if triple.contains("linux") {
            Self::Linux
        } else {
            Self::Other
        }
    }

    const fn is_windows(self) -> bool {
        matches!(self, Self::WindowsMsvc | Self::WindowsGnu)
    }
}

// ---------------------------------------------------------------------------
// Build feature flags (queried once from CARGO_CFG_TARGET + crate features)
// ---------------------------------------------------------------------------

struct Features {
    common: bool,
    cuda: bool,
    cuda_no_vmm: bool,
    metal: bool,
    vulkan: bool,
    rocm: bool,
    openmp: bool,
    dynamic_link: bool,
    dynamic_backends: bool,
    system_ggml: bool,
    mtmd: bool,
    llguidance: bool,
    shared_stdcxx: bool,
    static_stdcxx: bool,
}

impl Features {
    fn from_env() -> Self {
        let env_feature = |k: &str| env::var(format!("CARGO_FEATURE_{k}")).is_ok();
        Self {
            common: env_feature("COMMON"),
            cuda: env_feature("CUDA"),
            cuda_no_vmm: env_feature("CUDA_NO_VMM"),
            metal: env_feature("METAL"),
            vulkan: env_feature("VULKAN"),
            rocm: env_feature("ROCM"),
            openmp: env_feature("OPENMP"),
            dynamic_link: env_feature("DYNAMIC_LINK"),
            dynamic_backends: env_feature("DYNAMIC_BACKENDS"),
            system_ggml: env_feature("SYSTEM_GGML"),
            mtmd: env_feature("MTMD"),
            llguidance: env_feature("LLGUIDANCE"),
            shared_stdcxx: env_feature("SHARED_STDCXX"),
            static_stdcxx: env_feature("STATIC_STDCXX"),
        }
    }
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_dir = out_dir
        .ancestors()
        .nth(3) // .../target/<profile>/build/<pkg>-<hash>/out
        .map_or_else(|| out_dir.clone(), PathBuf::from);

    let features = Features::from_env();
    let os = TargetOs::from_env();

    // Trigger a rebuild whenever the wrapper headers, our own build.rs or any
    // tracked llama.cpp source file changes.
    watch_sources(&manifest_dir);

    // 1. Run bindgen first so the `OUT_DIR/bindings.rs` file exists even if
    //    the CMake build is going to be a no-op (e.g. dynamic-link).
    let bindings = run_bindgen(&manifest_dir, &features);

    // 2. Build the C/C++ libraries.
    let lib_paths = if features.dynamic_link {
        // Skip building; expect the user to provide the libs at runtime.
        Vec::new()
    } else {
        build_llama_cpp(&manifest_dir, &out_dir, &features, os)
    };

    // 3. Compile the C++ wrapper objects (chat, oaicompat, mtmd helpers).
    if !features.dynamic_link {
        build_cpp_wrappers(&manifest_dir, &out_dir, &features);
    }

    // 4. Emit cargo directives for include dirs and link flags.
    emit_link_directives(&manifest_dir, &out_dir, &target_dir, &features, os, &lib_paths);

    // 5. Write bindings to OUT_DIR.
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("writing bindings.rs failed");
}

// ---------------------------------------------------------------------------
// Source file watcher
// ---------------------------------------------------------------------------

fn watch_sources(manifest_dir: &PathBuf) {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=wrappers/");

    let llcpp = manifest_dir.join("llama.cpp");
    if llcpp.is_dir() {
        for sub in ["include", "src", "common", "tools/mtmd", "ggml/include"] {
            let dir = llcpp.join(sub);
            if !dir.is_dir() {
                continue;
            }
            for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
                let p = entry.path();
                if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                    if matches!(ext, "h" | "hpp" | "c" | "cpp" | "cxx" | "m" | "mm" | "metal") {
                        println!("cargo:rerun-if-changed={}", p.display());
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Bindgen invocation
// ---------------------------------------------------------------------------

fn run_bindgen(manifest_dir: &PathBuf, features: &Features) -> bindgen::Bindings {
    let llcpp = manifest_dir.join("llama.cpp");
    let mut builder = bindgen::Builder::default()
        .header(manifest_dir.join("wrapper.h").display().to_string())
        .allowlist_function("ggml_.*")
        .allowlist_function("gguf_.*")
        .allowlist_function("llama_.*")
        .allowlist_type("ggml_.*")
        .allowlist_type("gguf_.*")
        .allowlist_type("llama_.*")
        .allowlist_var("GGML_.*")
        .allowlist_var("LLAMA_.*")
        .allowlist_var("llama_.*")
        .derive_partialeq(true)
        .derive_eq(true)
        .derive_hash(true)
        .derive_debug(true)
        .prepend_enum_name(false)
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .clang_arg(format!("-I{}", llcpp.join("include").display()))
        .clang_arg(format!("-I{}", llcpp.join("ggml/include").display()))
        .clang_arg("-DFN_HEADER_DISABLED")
        .clang_arg("-DGGML_BACKEND_DL_DISABLE");

    if features.common {
        builder = builder
            .clang_arg(format!("-I{}", llcpp.join("common").display()))
            .allowlist_function("llama_rs_.*")
            .allowlist_type("llama_rs_.*");
    }

    if features.mtmd {
        builder = builder
            .clang_arg(format!("-I{}", llcpp.join("tools/mtmd").display()))
            .allowlist_function("mtmd_.*")
            .allowlist_type("mtmd_.*");
    }

    if features.llguidance {
        // llguidance is wired via a custom C-ABI vtable; we only need a stub.
        builder = builder
            .clang_arg("-DLLGUIDANCE_ENABLED")
            .allowlist_function("llg_.*");
    }

    if cfg!(target_os = "macos") {
        builder = builder
            .clang_arg("-D__ARM_FEATURE_NEON")
            .clang_arg("-DGGML_USE_METAL");
    }

    // macOS: use framework clang
    if cfg!(target_os = "macos") {
        builder = builder
            .clang_arg("-x")
            .clang_arg("c++")
            .clang_arg("-std=c++17")
            .clang_arg("-isysroot")
            .clang_arg(
                Command::new("xcrun")
                    .args(["--sdk", "macosx", "--show-sdk-path"])
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map_or_else(String::new, |s| s.trim().to_string()),
            );
    }

    builder
        .generate()
        .expect("bindgen failed to generate bindings")
}

// ---------------------------------------------------------------------------
// CMake build of llama.cpp
// ---------------------------------------------------------------------------

fn build_llama_cpp(
    manifest_dir: &PathBuf,
    out_dir: &PathBuf,
    features: &Features,
    os: TargetOs,
) -> Vec<PathBuf> {
    let llcpp = manifest_dir.join("llama.cpp");
    if !llcpp.is_dir() {
        panic!(
            "llama.cpp submodule missing. Run: git submodule update --init --recursive"
        );
    }

    // Stale build dirs can cause a lot of grief across host/target changes.
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let cmake_build = out_dir.join("llama.cpp");
    if cmake_build.is_dir() {
        let _ = std::fs::remove_dir_all(&cmake_build);
    }

    let mut dst = Config::new(&llcpp);
    dst.profile(profile_to_cmake(&profile));
    dst.define("LLAMA_BUILD_TESTS", "OFF");
    dst.define("LLAMA_BUILD_EXAMPLES", "OFF");
    dst.define("LLAMA_BUILD_SERVER", "OFF");
    dst.define("LLAMA_BUILD_TOOLS", "OFF");
    dst.define("LLAMA_BUILD_APP", "OFF");
    dst.define("LLAMA_BUILD_COMMON", "OFF");
    dst.define("LLAMA_CURL", "OFF");
    dst.define("LLAMA_USE_PREBUILT_UI", "OFF");
    dst.define("LLAMA_BUILD_UI", "OFF");
    dst.define("BUILD_SHARED_LIBS", if features.dynamic_link { "ON" } else { "OFF" });
    dst.define("GGML_NATIVE", "OFF"); // let the user control via RUSTFLAGS

    // CPU feature auto-detect
    let target_cpu = env::var("CARGO_CFG_TARGET_CPU").unwrap_or_default();
    if target_cpu == "native" {
        dst.define("GGML_NATIVE", "ON");
    } else {
        detect_cpu_features(&mut dst, &target_cpu);
    }

    // Backends
    if features.cuda {
        dst.define("GGML_CUDA", "ON");
        if features.cuda_no_vmm {
            dst.define("GGML_CUDA_NO_VMM", "ON");
        }
    } else {
        dst.define("GGML_CUDA", "OFF");
    }
    dst.define("GGML_METAL", if features.metal { "ON" } else { "OFF" });
    dst.define("GGML_VULKAN", if features.vulkan { "ON" } else { "OFF" });
    dst.define("GGML_HIP", if features.rocm { "ON" } else { "OFF" });
    dst.define("GGML_OPENMP", if features.openmp { "ON" } else { "OFF" });
    dst.define(
        "GGML_BACKEND_DL",
        if features.dynamic_backends { "ON" } else { "OFF" },
    );

    // Static vs dynamic C++ runtime on Android
    if os == TargetOs::Android {
        if features.shared_stdcxx {
            dst.define("ANDROID_STL", "c++_shared");
        } else if features.static_stdcxx {
            dst.define("ANDROID_STL", "c++_static");
        } else {
            dst.define("ANDROID_STL", "c++_static");
        }
    }

    // Build
    let install = dst.build();

    // Discover built libraries and tell cargo about them.
    discover_libs(&install)
}

fn profile_to_cmake(p: &str) -> &'static str {
    match p {
        "debug" => "Debug",
        "release" => "Release",
        "release-with-debug" => "RelWithDebInfo",
        _ => "Release",
    }
}

fn detect_cpu_features(dst: &mut Config, cpu: &str) {
    // Mapping from Rust `target-cpu` values to the GGML compile flags.
    // This is a simplified subset of the table in llama-cpp-rs/llama-cpp-sys-2/build.rs.
    let defines: &[(&str, &[&str])] = &[
        ("x86_64", &[]),
        ("sandybridge", &["GGML_SSE42", "GGML_AVX"]),
        ("haswell", &["GGML_SSE42", "GGML_AVX", "GGML_AVX2", "GGML_FMA", "GGML_F16C"]),
        ("skylake", &["GGML_SSE42", "GGML_AVX", "GGML_AVX2", "GGML_FMA", "GGML_F16C"]),
        ("skylake-avx512", &["GGML_SSE42", "GGML_AVX", "GGML_AVX2", "GGML_AVX512", "GGML_FMA", "GGML_F16C"]),
        ("icelake-client", &["GGML_SSE42", "GGML_AVX", "GGML_AVX2", "GGML_AVX512", "GGML_FMA", "GGML_F16C"]),
        ("apple-m1", &["GGML_SSE42", "GGML_AVX", "GGML_AVX2", "GGML_FMA", "GGML_F16C"]),
        ("apple-m2", &["GGML_SSE42", "GGML_AVX", "GGML_AVX2", "GGML_FMA", "GGML_F16C"]),
        ("apple-m3", &["GGML_SSE42", "GGML_AVX", "GGML_AVX2", "GGML_FMA", "GGML_F16C"]),
    ];

    for (key, defs) in defines {
        if key == &cpu {
            for d in *defs {
                dst.define(*d, "ON");
            }
            return;
        }
    }
    // Fallback: assume x86_64 baseline.
    let _ = dst; // no-op
}

fn discover_libs(install: &PathBuf) -> Vec<PathBuf> {
    let lib_dir = install.join("lib");
    let build_dir = install.join("build");
    let mut libs = Vec::new();
    let pattern = if cfg!(target_os = "windows") {
        "*.lib"
    } else {
        "*.a"
    };
    for dir in [&lib_dir, &build_dir] {
        if !dir.is_dir() {
            continue;
        }
        for entry in glob(&dir.join(pattern).display().to_string()).unwrap() {
            libs.push(entry.unwrap());
        }
    }
    libs
}

// ---------------------------------------------------------------------------
// C++ wrapper compilation (chat templates, oaicompat, mtmd helpers, llguidance)
// ---------------------------------------------------------------------------

fn build_cpp_wrappers(manifest_dir: &PathBuf, out_dir: &PathBuf, features: &Features) {
    use cc::Build;

    let llcpp = manifest_dir.join("llama.cpp");
    let mut build = Build::new();
    build
        .cpp(true)
        .std("c++17")
        .include(llcpp.join("include"))
        .include(llcpp.join("common"))
        .include(llcpp.join("ggml/include"))
        .include(manifest_dir.join("wrappers"))
        .warnings(false)
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-function")
        .flag_if_supported("-Wno-deprecated-declarations");

    let mut file_count = 0_usize;
    // Always compile the wrapper shim entry point so the static library
    // exists even when no optional feature is enabled.
    let shim = manifest_dir.join("wrappers/mod.cpp");
    if shim.is_file() {
        build.file(shim);
        file_count += 1;
    }
    if features.common {
        let oai = manifest_dir.join("wrappers/oaicompat.cpp");
        if oai.is_file() {
            build.file(oai);
            file_count += 1;
        }
        let grammar = manifest_dir.join("wrappers/grammar.cpp");
        if grammar.is_file() {
            build.file(grammar);
            file_count += 1;
        }
    }
    if features.mtmd {
        let path = manifest_dir.join("wrappers/mtmd_helpers.cpp");
        if path.is_file() {
            build.file(path);
            file_count += 1;
        }
    }
    if features.llguidance {
        let path = manifest_dir.join("wrappers/llguidance_vtable.cpp");
        if path.is_file() {
            build.file(path);
            file_count += 1;
        }
    }

    if file_count > 0 {
        build.compile("llama_crab_sys_wrappers");
    }

    // Re-touch the OUT_DIR marker so cargo sees the result.
    let _ = std::fs::write(out_dir.join("wrappers.built"), b"ok");
}

// ---------------------------------------------------------------------------
// Cargo link directives
// ---------------------------------------------------------------------------

fn emit_link_directives(
    manifest_dir: &PathBuf,
    out_dir: &PathBuf,
    _target_dir: &PathBuf,
    features: &Features,
    os: TargetOs,
    lib_paths: &[PathBuf],
) {
    let llcpp = manifest_dir.join("llama.cpp");

    // Include directories
    println!("cargo:include={}", llcpp.join("include").display());
    println!("cargo:include={}", llcpp.join("ggml/include").display());
    if features.common {
        println!("cargo:include={}", llcpp.join("common").display());
    }
    if features.mtmd {
        println!("cargo:include={}", llcpp.join("tools/mtmd").display());
    }

    // Library search paths and library names
    if !features.dynamic_link {
        // Collect distinct search directories from the discovered libs.
        let mut search_dirs: Vec<PathBuf> = lib_paths
            .iter()
            .map(|p| p.parent().map(PathBuf::from).unwrap_or_else(|| p.clone()))
            .collect();
        search_dirs.sort();
        search_dirs.dedup();
        for dir in &search_dirs {
            println!("cargo:rustc-link-search=native={}", dir.display());
        }

        // Discover which lib names actually have a static archive in one of
        // the search directories and emit a link directive for each.
        let mut seen = std::collections::BTreeSet::new();
        for lib in [
            "llama",
            "ggml",
            "ggml-cpu",
            "ggml-base",
            "ggml-blas",
            "ggml-metal",
            "common",
            "mtmd",
        ] {
            if seen.insert(lib.to_string()) && lib_present_in_paths(lib, &search_dirs) {
                println!("cargo:rustc-link-lib=static={lib}");
            }
        }
    }

    // External C++ libs
    match os {
        TargetOs::Macos => {
            println!("cargo:rustc-link-lib=framework=Foundation");
            println!("cargo:rustc-link-lib=framework=Metal");
            println!("cargo:rustc-link-lib=framework=MetalKit");
            println!("cargo:rustc-link-lib=framework=Accelerate");
            println!("cargo:rustc-link-lib=c++");
        }
        TargetOs::Linux => {
            println!("cargo:rustc-link-lib=stdc++");
            if features.openmp {
                println!("cargo:rustc-link-lib=gomp");
            }
        }
        TargetOs::WindowsMsvc => {
            println!("cargo:rustc-link-lib=advapi32");
        }
        TargetOs::WindowsGnu => {
            println!("cargo:rustc-link-lib=stdc++");
        }
        TargetOs::Android => {
            if features.shared_stdcxx {
                println!("cargo:rustc-link-lib=c++_shared");
            } else {
                println!("cargo:rustc-link-lib=c++_static");
                println!("cargo:rustc-link-lib=c++abi");
            }
            println!("cargo:rustc-link-lib=log");
            println!("cargo:rustc-link-lib=android");
        }
        TargetOs::Other => {}
    }

    if features.cuda && os == TargetOs::Linux {
        println!("cargo:rustc-link-lib=cudart_static");
        println!("cargo:rustc-link-lib=cublas_static");
        println!("cargo:rustc-link-lib=cublasLt_static");
        println!("cargo:rustc-link-lib=culibos");
    }
    if features.cuda && os == TargetOs::WindowsMsvc {
        println!("cargo:rustc-link-lib=cudart");
        println!("cargo:rustc-link-lib=cublas");
        println!("cargo:rustc-link-lib=cublasLt");
        println!("cargo:rustc-link-lib=cuda");
    }
    if features.rocm {
        println!("cargo:rustc-link-lib=amdhip64");
        println!("cargo:rustc-link-lib=rocblas");
        println!("cargo:rustc-link-lib=hipblas");
    }
    if features.vulkan {
        if os == TargetOs::WindowsMsvc {
            println!("cargo:rustc-link-lib=vulkan-1");
        } else {
            println!("cargo:rustc-link-lib=vulkan");
        }
    }

    // Compile out_dir as a side effect so cargo knows to keep it.
    let _ = out_dir;
}

fn lib_present_in_paths(name: &str, dirs: &[PathBuf]) -> bool {
    let prefixes: &[&str] = &["lib", ""];
    let suffixes: &[&str] = &[".a", ".lib", ".so", ".dylib", ".dll"];
    for d in dirs {
        for pre in prefixes {
            for suf in suffixes {
                if d.join(format!("{pre}{name}{suf}")).exists() {
                    return true;
                }
            }
        }
    }
    false
}
