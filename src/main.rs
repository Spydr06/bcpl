#![feature(let_chains)]

use std::collections::HashSet;

use crate::source_file::{SourceFile, SourceFileId};
use crate::context::{Context, BuildKind, OutputFile};

mod context;
mod source_file;
mod token;
mod ast;
mod parser;
mod error;

trait ExpectArg<T> {
    fn expect_arg(self, program_name: &str, arg: &str) -> T;
}

impl<T> ExpectArg<T> for Option<T> {
    fn expect_arg(self, program_name: &str, arg: &str) -> T {
        match self {
            None => {
                eprintln!("{program_name}: option requires an argument -- '{arg}'");
                eprintln!("Try `{program_name} --help` for more information.");
                std::process::exit(1);
            }
            Some(val) => val
        }
    }
}

fn main() {
    let mut args = std::env::args();
    let mut ctx = Context::from_program_name(args.next().expect("Error getting program name"));

    let mut input_files = HashSet::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => help(ctx.program_name()),
            "-o" => ctx.set_output_file(args.next().expect_arg(ctx.program_name(), arg.as_str())),
            "-D" => ctx.define_tag(args.next().expect_arg(ctx.program_name(), arg.as_str())),
            "-c" => ctx.set_build_kind(BuildKind::Object),
            "--shared" => ctx.set_build_kind(BuildKind::SharedObject),
            _ if arg.starts_with("-") => {
                eprintln!("{}: invalid option -- {}", ctx.program_name(), arg);
                eprintln!("Try `{} --help` for more information.", ctx.program_name());
            }
            _ => {
                input_files.insert(arg);
            }
        }
    }

    ctx.add_source_files(input_files.into_iter()
        .enumerate()
        .map(|(id, path)| (id as SourceFileId, SourceFile::read(path, id as SourceFileId).expect("error opening file")))
        .collect()
    );

    if let Err(_) = ctx.compile() {
        terminate()
    }
}

fn usage(program_name: &str) {
    println!("Usage: {program_name} <input file> [OPTIONS]\n");
}

fn help(program_name: &str) -> ! {
    usage(program_name);

    println!("Options:
  -o <output file>  Set an output file; default: `{}`
  -D <tag name>     Set a BCPL tag.
  -c                Skip linking and emit `.o` file.
  --shared          Create a shared library.
  -h, --help        Print this help text and exit.",
    OutputFile::default().to_filename(&BuildKind::default())); 

    std::process::exit(0);
}

fn terminate() -> ! {
    println!("compilation terminated.");
    std::process::exit(1);
}

