use phf::phf_map;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Lang {
    pub plugin_name: &'static str,
    pub display_name: &'static str,
    pub compile_command: &'static [&'static str],
    pub run_command: &'static [&'static str],
    pub plugin: &'static str,
    pub env: &'static [(&'static str, &'static str)],
    pub install_env: &'static [(&'static str, &'static str)],
    pub latest_version: &'static str,
    pub icon: &'static str,
}

pub const LANGS: phf::Map<&'static str, Lang> = phf_map! {
    "nodejs" => Lang {
        plugin_name: "nodejs",
        display_name: "JavaScript (NodeJS)",
        compile_command: &[],
        run_command: &["${LANG_LOCATION}/bin/node", "${FILE_LOCATION}"],
        plugin: "https://github.com/asdf-vm/asdf-nodejs.git",
        env: &[],
        install_env: &[],
        latest_version: "22.9.0",
        icon: "nodejs.svg"
    },
    "deno" => Lang {
        plugin_name: "deno",
        display_name: "JavaScript (Deno)",
        compile_command: &[],
        run_command: &["${LANG_LOCATION}/bin/deno", "--allow-write=/tmp", "--allow-run", "--allow-read", "${FILE_LOCATION}"],
        //run_command: &["/usr/bin/env"],
        plugin: "https://github.com/asdf-community/asdf-deno.git",
        env: &[
            ("RUST_BACKTRACE", "1"),
            ("NO_COLOR", "1")
        ],
        install_env: &[],
        latest_version: "2.0.6",
        icon: "deno.svg"
    },
    "python" => Lang {
        plugin_name: "python",
        display_name: "Python",
        compile_command: &[],
        run_command: &["${LANG_LOCATION}/bin/python", "${FILE_LOCATION}"],
        plugin: "https://github.com/asdf-community/asdf-python.git",
        env: &[("LD_LIBRARY_PATH", "/lang/lib")],
        install_env: &[],
        latest_version: "3.12.0",
        icon: "python.svg"
    },
    "rust" => Lang {
        plugin_name: "rust",
        display_name: "Rust",
        compile_command: &["${LANG_LOCATION}/bin/rustc", "${FILE_LOCATION}", "-o", "${OUTPUT_LOCATION}"],
        run_command: &["${OUTPUT_LOCATION}"],
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
        icon: "rust.svg"
    },
    "vyxal" => Lang {
        plugin_name: "vyxal",
        display_name: "Vyxal",
        compile_command: &[],
        run_command: &["${LANG_LOCATION}/bin/vyxal2", "${FILE_LOCATION}", "'□'"],
        plugin: "https://github.com/lyxal/vyxasdf.git",
        env: &[],
        install_env: &[],
        latest_version: "2.22.4.3",
        icon: "vyxal.svg"
    },
    "vyxal3" => Lang {
        plugin_name: "vyxal3",
        display_name: "Vyxal 3",
        compile_command: &[],
        run_command: &["${LANG_LOCATION}/bin/vyxal3", "${FILE_LOCATION}", "--stdin"],
        plugin: "https://github.com/lyxal/vyxasd3f.git",
        env: &[],
        install_env: &[],
        latest_version: "3.5.1",
        icon: "vyxal3.svg"
    },
    "tinyapl" => Lang {
        plugin_name: "tinyapl",
        display_name: "APL (TinyAPL)",
        compile_command: &[],
        run_command: &["${LANG_LOCATION}/bin/tinyapl", "${FILE_LOCATION}"],
        plugin: "https://github.com/RubenVerg/asdf-tinyapl.git",
        env: &[],
        install_env: &[],
        latest_version: "0.11.1.0",
        icon: "tinyapl.svg"
    },
    "tcc" => Lang {
        plugin_name: "tcc",
        display_name: "C (tcc)",
        compile_command: &[],
        run_command: &["${LANG_LOCATION}/bin/tcc", "-run", "-B", "${LANG_LOCATION}/lib/tcc", "${FILE_LOCATION}"],
        plugin: "https://github.com/mousetail/asdf-plugin-tcc.git",
        env: &[
            ("C_INCLUDE_PATH", "{LANG_LOCATION}/include"),
            ("LIBRARY_PATH", "${LANG_LOCATION}/lib")
        ],
        install_env: &[],
        latest_version: "0.9.27",
        icon: "c.svg"
    }
};
