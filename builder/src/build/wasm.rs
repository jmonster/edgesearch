use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

const RUNNER_C_BITSET: &'static str = include_str!("../../resources/bitset.c");
const RUNNER_C_BLOOM: &'static str = include_str!("../../resources/bloom.c");
const RUNNER_C_MAIN: &'static str = include_str!("../../resources/main.c");
const RUNNER_C_MURMUR: &'static str = include_str!("../../resources/murmur.c");
const RUNNER_C_ROARING: &'static str = include_str!("../../resources/roaring.c");
const RUNNER_C_SORT: &'static str = include_str!("../../resources/sort.c");
const RUNNER_C_SYS: &'static str = include_str!("../../resources/sys.c");

pub enum WasmStandard {
    C89,
    C99,
    C11,
    C17,
}

pub enum WasmOptimisationLevel {
    Level(u8),
    Fast,
    S,
    Z,
    G,
}

pub struct WasmCompileArgs<'m, 'i, 'o> {
    standard: WasmStandard,
    optimisation_level: WasmOptimisationLevel,
    all_warnings: bool,
    extra_warnings: bool,
    warnings_as_errors: bool,
    macros: &'m [(&'m str, &'m str)],
    input: &'i PathBuf,
    output: &'o PathBuf,
}

pub fn compile_to_wasm(WasmCompileArgs {
    standard,
    optimisation_level,
    all_warnings,
    extra_warnings,
    warnings_as_errors,
    macros,
    input,
    output,
}: WasmCompileArgs) -> () {
    let mut cmd = Command::new("clang");
    cmd.arg(format!("-std={}", match standard {
        WasmStandard::C89 => "c89",
        WasmStandard::C99 => "c99",
        WasmStandard::C11 => "c11",
        WasmStandard::C17 => "c17",
    }).as_str());
    cmd.arg(format!("-O{}", match optimisation_level {
        WasmOptimisationLevel::Level(0) => "0",
        WasmOptimisationLevel::Level(1) => "1",
        WasmOptimisationLevel::Level(2) => "2",
        WasmOptimisationLevel::Level(3) => "3",
        WasmOptimisationLevel::Fast => "fast",
        WasmOptimisationLevel::S => "s",
        WasmOptimisationLevel::Z => "z",
        WasmOptimisationLevel::G => "g",
        _ => panic!("Invalid optimisation level"),
    }).as_str());
    if all_warnings { cmd.arg("-Wall"); };
    if extra_warnings { cmd.arg("-Wextra"); };
    if warnings_as_errors { cmd.arg("-Werror"); };
    cmd.arg("--target=wasm32-unknown-unknown-wasm")
        .arg("-nostdlib")
        .arg("-nostdinc")
        .arg("-isystemstubs")
        .arg("-Wl,--no-entry")
        .arg("-Wl,--import-memory");
    for (name, code) in macros.iter() {
        cmd.arg(format!("-D{}={}", name, code));
    };
    cmd.arg(input);
    cmd.arg("-o").arg(output);

    let result = cmd.status().expect("compile WASM");
    if !result.success() {
        panic!("Failed to compile WASM");
    };
}

pub fn generate_and_compile_runner_wasm(output_dir: &PathBuf, max_results: usize, max_query_bytes: usize) -> () {
    let source_path = output_dir.join("runner.c");
    let output_path = output_dir.join("runner.wasm");

    let mut source_file = File::create(&source_path).expect("open runner.c for writing");
    source_file.write_all(RUNNER_C_SYS.as_bytes()).expect("write runner.c");
    source_file.write_all(RUNNER_C_MAIN.as_bytes()).expect("write runner.c");
    source_file.write_all(RUNNER_C_MURMUR.as_bytes()).expect("write runner.c");
    source_file.write_all(RUNNER_C_SORT.as_bytes()).expect("write runner.c");
    source_file.write_all(RUNNER_C_ROARING.as_bytes()).expect("write runner.c");
    source_file.write_all(RUNNER_C_BITSET.as_bytes()).expect("write runner.c");
    source_file.write_all(RUNNER_C_BLOOM.as_bytes()).expect("write runner.c");

    compile_to_wasm(WasmCompileArgs {
        standard: WasmStandard::C11,
        optimisation_level: WasmOptimisationLevel::Level(3),
        all_warnings: true,
        extra_warnings: true,
        warnings_as_errors: false,
        macros: &[
            ("MAX_RESULTS", format!("{}", max_results).as_str()),
            ("MAX_QUERY_BYTES", format!("{}", max_query_bytes).as_str()),
        ],
        input: &source_path,
        output: &output_path,
    });
}
