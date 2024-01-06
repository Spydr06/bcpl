#![feature(let_chains)]
#![feature(impl_trait_in_assoc_type)]
#![feature(trait_alias)]

use std::collections::{HashSet, HashMap};

use colorize::AnsiColor;
use source_file::Located;

use crate:: {
    error::CompilerError,
    source_file::{SourceFile, SourceFileId},
    context::{Context, BuildKind, OutputFile},
};

mod context;
mod source_file;
mod token;
mod ast;
mod parser;
mod error;
mod typechecker;

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

    use context::CompileResult as C;
    match ctx.compile() {
        C::Ok => (),
        C::Warn(warns) => warns.into_iter().for_each(|warn| highlight_error(warn, ctx.source_files())),
        C::Err(errors) => {
            errors.into_iter().for_each(|err| highlight_error(err, ctx.source_files()));
            terminate()
        }
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
    
fn highlight_error(err: Located<CompilerError>, source_files: &HashMap<SourceFileId, SourceFile>) {
    let loc = err.location();
    let file = source_files.get(&loc.file_id()).expect("invalid file id");

    println!("{} {}:{}:{}: {}", err.severity(), file.path(), loc.line(), loc.column(), err.message());
    print!("{} {} ", format!(" {: >4}", loc.line()).bold().b_black(), "|".b_black());

    let line = file.line(loc.line()).unwrap();
    let mark_start = loc.column();
    let mark_end = loc.column() + loc.width();
    println!("{}{}{}", &line[..mark_start], (&line[mark_start..mark_end]).to_owned().bold().b_yellow(), &line[mark_end..]);

    print!("      {} {}{}", "|".b_black(), " ".repeat(mark_start), "~".repeat(loc.width()).yellow());

    if let Some(hint) = err.hint() {
        print!(" {} {} {}", "<-".b_black(), "hint:".bold().b_grey(), hint.clone().b_grey());
    }

    println!();

    for additional in &err.additional {
        highlight_error(additional.clone().clone(), source_files); 
    }
}
 
