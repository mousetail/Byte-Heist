use phf::phf_map;
use serde::Serialize;

#[derive(Serialize, Default)]
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
    pub extra_mounts: &'static [(&'static str, &'static str)],
    pub extension: &'static str,
    pub extra_runtime: u64,
}

pub const DEFAULT_LANG: Lang = Lang {
    plugin_name: "",
    display_name: "",
    compile_command: &[],
    run_command: &[],
    plugin: "",
    env: &[],
    install_env: &[],
    latest_version: "",
    icon: "",
    extra_mounts: &[],
    extension: "",
    extra_runtime: 0,
};

pub const LANGS: phf::Map<&'static str, Lang> = phf_map! {
    "nodejs" => Lang {
        plugin_name: "nodejs",
        display_name: "JavaScript (NodeJS)",
        run_command: &["${LANG_LOCATION}/bin/node", "${FILE_LOCATION}"],
        plugin: "https://github.com/asdf-vm/asdf-nodejs.git",
        latest_version: "22.9.0",
        icon: "nodejs.svg",
        ..DEFAULT_LANG
    },
    "deno" => Lang {
        plugin_name: "deno",
        display_name: "JavaScript (Deno)",
        run_command: &["${LANG_LOCATION}/bin/deno", "--allow-write=/tmp", "--allow-run", "--allow-read", "${FILE_LOCATION}"],
        plugin: "https://github.com/asdf-community/asdf-deno.git",
        env: &[
            ("RUST_BACKTRACE", "1"),
            ("NO_COLOR", "1")
        ],
        latest_version: "2.0.6",
        icon: "deno.svg",
        ..DEFAULT_LANG
    },
    "python" => Lang {
        plugin_name: "python",
        display_name: "Python",
        compile_command: &[],
        run_command: &["${LANG_LOCATION}/bin/python", "${FILE_LOCATION}"],
        plugin: "https://github.com/asdf-community/asdf-python.git",
        env: &[("LD_LIBRARY_PATH", "/lang/lib")],
        latest_version: "3.12.0",
        icon: "python.svg",
        ..DEFAULT_LANG
    },
    "rust" => Lang {
        plugin_name: "rust",
        display_name: "Rust",
        compile_command: &["${LANG_LOCATION}/bin/rustc", "${FILE_LOCATION}", "-o", "${OUTPUT_LOCATION}", "--edition", "2024"],
        run_command: &["${OUTPUT_LOCATION}"],
        plugin: "https://github.com/asdf-community/asdf-rust.git",
        env: &[
            ("LD_LIBRARY_PATH", "/usr/libexec/gcc/x86_64-linux-gnu/14:/usr/lib:/lang/lib:/lib"),
            ("PATH", "/usr/bin:/bin")
        ],
        install_env: &[(
            "RUST_WITHOUT",
            "rust-docs,rust-docs-json-preview,cargo,rustfmt-preview,rls-preview,rust-analyzer-preview,llvm-tools-preview,clippy-preview,rust-analysis-x86_64-unknown-linux-gnu,llvm-bitcode-linker-preview"
        )],
        latest_version: "1.85.0",
        icon: "rust.svg",
        extra_mounts: &[
            ("/usr/bin/x86_64-linux-gnu-gcc-14", "/usr/bin/cc"),
            ("/usr/bin/x86_64-linux-gnu-ld.bfd", "/usr/bin/ld"),
            ("/usr/libexec/gcc/x86_64-linux-gnu/14", "/usr/libexec/gcc/x86_64-linux-gnu/14"),
            //("/usr/lib/gcc/x86_64-linux-gnu/14/", "/usr/lib/gcc/x86_64-linux-gnu/14/")
        ],
        ..DEFAULT_LANG
    },
    "vyxal" => Lang {
        plugin_name: "vyxal",
        display_name: "Vyxal",
        run_command: &["${LANG_LOCATION}/bin/vyxal2", "${FILE_LOCATION}", "'□'"],
        plugin: "https://github.com/lyxal/vyxasdf.git",
        latest_version: "2.22.4.3",
        icon: "vyxal.svg",
        extra_runtime: 2,
        ..DEFAULT_LANG
    },
    "vyxal3" => Lang {
        plugin_name: "vyxal3",
        display_name: "Vyxal 3",
        run_command: &["/java/bin/java", "-jar", "${LANG_LOCATION}/bin/vyxal3.jar", "--file", "${FILE_LOCATION}", "--stdin"],
        plugin: "https://github.com/lyxal/vyxasd3f.git",
        env: &[
            ("LD_LIBRARY_PATH", "/java/lib:/lib"),
            ("JAVA_TOOL_OPTIONS", "-Dfile.encoding=UTF-8")
        ],
        latest_version: "3.7.0",
        icon: "vyxal3.svg",
        extra_mounts: &[
            ("/usr/lib/jvm/java-17-openjdk-amd64", "/java", )
        ],
        extra_runtime: 2,
        ..DEFAULT_LANG
    },
    "tinyapl" => Lang {
        plugin_name: "tinyapl",
        display_name: "APL (TinyAPL)",
        run_command: &["${LANG_LOCATION}/bin/tinyapl", "${FILE_LOCATION}"],
        plugin: "https://github.com/RubenVerg/asdf-tinyapl.git",
        latest_version: "0.12.0.0",
        icon: "tinyapl.svg",
        ..DEFAULT_LANG
    },
    "tcc" => Lang {
        plugin_name: "tcc",
        display_name: "C (tcc)",
        run_command: &["${LANG_LOCATION}/bin/tcc", "-run", "-B", "${LANG_LOCATION}/lib/tcc", "${FILE_LOCATION}"],
        plugin: "https://github.com/mousetail/asdf-plugin-tcc.git",
        env: &[
            ("C_INCLUDE_PATH", "{LANG_LOCATION}/include"),
            ("LIBRARY_PATH", "${LANG_LOCATION}/lib")
        ],
        latest_version: "0.9.27",
        icon: "c.svg",
        ..DEFAULT_LANG
    },
    // "kotlin" => Lang {
    //     plugin_name: "kotlin",
    //     display_name: "Kotlin",
    //     compile_command: &["${LANG_LOCATION}/kotlinc/bin/kotlinc", "${FILE_LOCATION}", "-include-runtime", "-d", "${OUTPUT_LOCATION}.jar"],
    //     run_command: &["/java/bin/java", "-jar", "${OUTPUT_LOCATION}.jar"],
    //     plugin: "https://github.com/asdf-community/asdf-kotlin.git",
    //     env: &[
    //         ("LD_LIBRARY_PATH", "/java/lib:/lib"),
    //         ("JAVA_HOME", "/java")
    //     ],
    //     latest_version: "2.1.10",
    //     icon: "kotlin.svg",
    //     extra_mounts: &[
    //         ("/usr/lib/jvm/java-17-openjdk-amd64", "/java")
    //     ],
    //     extension: ".kt",
    //     extra_runtime: 2,
    //     ..DEFAULT_LANG
    // }
};
