use clap::{Parser as ClapParser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(ClapParser)]
#[command(name = "myc", version = "0.2.0", about = "Mycor compiler")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    /// file to run if no subcommand
    file: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    New {
        name: String,
    },
    Init,
    Run {
        file: Option<PathBuf>,
        #[arg(long)]
        interp: bool,
    },
    Build {
        #[arg(long)]
        release: bool,
    },
    Compile {
        file: PathBuf,
    },
    Check,
    Test,
    Fmt {
        files: Vec<PathBuf>,
    },
    Doc,
    Repl,
    EmitC {
        file: PathBuf,
    },
    Targets,
    Explain {
        code: String,
    },
    Fix,
    Clean,
    Bench,
    Version,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::New { name }) => {
            fs::create_dir_all(&name).unwrap();
            fs::write(
                format!("{}/main.myc", name),
                "fn main() {\n    println(\"hello, mycor\");\n}\n",
            )
            .unwrap();
            fs::write(
                format!("{}/mycor.toml", name),
                format!("[package]\nname = \"{}\"\nversion = \"0.1.0\"\n", name),
            )
            .unwrap();
            println!("created project {}", name);
        }
        Some(Commands::Init) => {
            fs::write(
                "main.myc",
                "fn main() {\n    println(\"hello, mycor\");\n}\n",
            )
            .unwrap();
            fs::write(
                "mycor.toml",
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\n",
            )
            .unwrap();
            println!("initialized");
        }
        Some(Commands::Run { file, interp }) => {
            let path = file.unwrap_or_else(|| PathBuf::from("main.myc"));
            let src = fs::read_to_string(&path).unwrap_or_default();
            if interp {
                let (f, _sm, _checker) =
                    myc_driver::compile_source(&src, path.to_str().unwrap_or("main.myc"));
                let mut interp = myc_interp::Interpreter::new(&f);
                if let Err(e) = interp.run_main() {
                    eprintln!("interp error: {}", e);
                }
            } else {
                let out = std::env::temp_dir().join("myc_run.exe");
                match myc_driver::build_executable(&src, path.to_str().unwrap_or("main.myc"), &out)
                {
                    Ok(exe) => {
                        let _ = myc_driver::run_executable(&exe);
                    }
                    Err(e) => {
                        eprintln!("error: {}", e);
                    }
                }
            }
        }
        Some(Commands::Build { release: _ }) => {
            let src = fs::read_to_string("main.myc").unwrap_or_default();
            let out = PathBuf::from("target/myc_app.exe");
            fs::create_dir_all("target").unwrap();
            match myc_driver::build_executable(&src, "main.myc", &out) {
                Ok(_) => println!("built {}", out.display()),
                Err(e) => eprintln!("error: {}", e),
            }
        }
        Some(Commands::Compile { file }) => {
            let src = fs::read_to_string(&file).unwrap_or_default();
            let out = file.with_extension("exe");
            match myc_driver::build_executable(&src, file.to_str().unwrap_or("main.myc"), &out) {
                Ok(_) => println!("compiled {}", out.display()),
                Err(e) => eprintln!("error: {}", e),
            }
        }
        Some(Commands::Check) => {
            let src = fs::read_to_string("main.myc").unwrap_or_default();
            let diags = myc_driver::check_source(&src, "main.myc");
            if diags.is_empty() {
                println!("check ok");
            } else {
                for d in diags {
                    println!("{:?}", d.code);
                }
            }
        }
        Some(Commands::EmitC { file }) => {
            let src = fs::read_to_string(&file).unwrap_or_default();
            let (f, _sm, checker) =
                myc_driver::compile_source(&src, file.to_str().unwrap_or("main.myc"));
            if !checker.diagnostics.is_empty() {
                for d in &checker.diagnostics {
                    eprintln!("{}", d.code);
                }
                return;
            }
            let c = myc_codegen_c::generate(&f, &checker.type_map);
            println!("{}", c);
        }
        Some(Commands::Targets) => {
            println!("x86_64-pc-windows-msvc");
            println!("x86_64-unknown-linux-gnu");
            println!("aarch64-apple-darwin");
        }
        Some(Commands::Explain { code }) => {
            let path = format!("docs/errors/{}.md", code);
            if let Ok(s) = fs::read_to_string(&path) {
                println!("{}", s);
            } else {
                println!("No documentation for {}", code);
            }
        }
        Some(Commands::Version) => println!("myc 0.2.0"),
        Some(Commands::Repl) => {
            println!("myc repl (not fully implemented)");
        }
        Some(Commands::Fmt { files }) => {
            let files = if files.is_empty() {
                vec![PathBuf::from("main.myc")]
            } else {
                files
            };
            for f in files {
                let src = fs::read_to_string(&f).unwrap_or_default();
                match myc_fmt::format_source(&src) {
                    Ok(formatted) => {
                        let _ = fs::write(&f, formatted);
                        println!("formatted {}", f.display());
                    }
                    Err(e) => eprintln!("fmt error: {}", e),
                }
            }
        }
        Some(Commands::Test) => {
            let (p, f) = myc_test_harness::run_all(std::path::Path::new("."));
            println!("tests: {} passed, {} failed", p, f);
        }
        Some(Commands::Doc) => {
            let src = fs::read_to_string("main.myc").unwrap_or_default();
            let (f, _sm, _checker) = myc_driver::compile_source(&src, "main.myc");
            let html = format!(
                "<html><body><pre>{}</pre></body></html>",
                myc_doc::summarize(&f)
            );
            let _ = fs::create_dir_all("target/doc");
            let _ = fs::write("target/doc/index.html", html);
            println!("docs written to target/doc");
        }
        Some(Commands::Fix) => {
            println!("myc fix: not yet implemented");
        }
        Some(Commands::Clean) => {
            let _ = fs::remove_dir_all("target");
            println!("cleaned");
        }
        Some(Commands::Bench) => {
            println!("myc bench: not yet implemented");
        }
        None => {
            if let Some(f) = cli.file {
                let src = fs::read_to_string(&f).unwrap_or_default();
                let out = std::env::temp_dir().join("myc_run.exe");
                match myc_driver::build_executable(&src, f.to_str().unwrap_or("main.myc"), &out) {
                    Ok(exe) => {
                        let _ = myc_driver::run_executable(&exe);
                    }
                    Err(e) => eprintln!("error: {}", e),
                }
            } else {
                println!("myc 0.2.0");
            }
        }
    }
}
