use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Lang {
    pub name: &'static str,
    pub compile_command: &'static [&'static str],
    pub run_command: &'static [&'static str],
    pub plugin: &'static str,
    pub env: &'static [(&'static str, &'static str)],
    pub install_env: &'static [(&'static str, &'static str)],
    pub latest_version: &'static str,
}

pub const LANGS: &[Lang] = &[
    Lang {
        name: "nodejs",
        compile_command: &[],
        run_command: &["${LANG_LOCATION}/bin/node", "${FILE_LOCATION}"],
        plugin: "https://github.com/asdf-vm/asdf-nodejs.git",
        env: &[],
        install_env: &[],
        latest_version: "22.9.0",
    },
    Lang {
        name: "deno",
        compile_command: &[],
        run_command: &["${LANG_LOCATION}/bin/deno", "--allow-write=/tmp", "--allow-run", "--allow-read", "${FILE_LOCATION}"],
        //run_command: &["/usr/bin/env"],
        plugin: "https://github.com/asdf-community/asdf-deno.git",
        env: &[
            ("RUST_BACKTRACE", "1")
        ],
        install_env: &[],
        latest_version: "2.0.4",
    },
    Lang {
        name: "python",
        compile_command: &[],
        run_command: &["${LANG_LOCATION}/bin/python", "${FILE_LOCATION}"],
        plugin: "https://github.com/asdf-community/asdf-python.git",
        env: &[("LD_LIBRARY_PATH", "/lang/lib")],
        install_env: &[],
        latest_version: "3.12.0",
    },
    Lang {
        name: "rust",
        compile_command: &["${LANG_LOCATION}/bin/rustc", "${FILE_LOCATION}", "-o", "/tmp/output"],
        run_command: &["/tmp/output"],
        plugin: "https://github.com/asdf-community/asdf-rust.git",
        env: &[
            ("LD_LIBRARY_PATH", "/lang/lib:/lib"),
            ("PATH", "/usr/bin:/bin")
        ],
        install_env: &[(
            "RUST_WITHOUT",
            "rust-docs,rust-docs-json-preview,cargo,rustfmt-preview,rls-preview,rust-analyzer-preview,llvm-tools-preview,clippy-preview,rust-analysis-x86_64-unknown-linux-gnu,llvm-bitcode-linker-preview"
        )],
        latest_version: "1.82.0",
    },
    Lang {
        name: "vyxal",
        compile_command: &[],
        run_command: &["${LANG_LOCATION}/bin/vyxal", "${FILE_LOCATION}"],
        plugin: "https://github.com/lyxal/vyxasdf.git",
        install_env: &[],
        latest_version: "2.22.0",
    },
];
