pub mod finders;
mod text_to_vec;
use std::path::Path;
mod vec_to_byte;
use std::{env, fs, process};
use text_to_vec::imports::import_from_file;
use text_to_vec::prepare_terrain::prepare_to_parse;

use crate::finders::find::{find_func, find_scopes};
use crate::vec_to_byte::public_values::global_things;

pub const RESERVED: [&str; 22] = [
    "if",
    "else",
    "match",
    "while",
    "for",
    "await",
    "do",
    "out",
    "inp",
    "public",
    "import",
    "let",
    "const",
    "Null",
    "Bool",
    "String",
    "Float",
    "Double",
    "MiniInt",
    "Int",
    "LongInt",
    "LongLongInt",
];

pub fn kill(msg: &str) -> ! {
    eprintln!("{}", msg);
    process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let (is_debug, path) = match args.get(1).map(|s| s.as_str()) {
        Some("--debug") => (
            true,
            args.get(2)
                .unwrap_or_else(|| kill("001 : NO_FILE_PROVIDED")),
        ),
        Some(p) => (false, &p.to_string()),
        None => kill("001 : NO_FILE_PROVIDED"),
    };

    let content = fs::read_to_string(&path).unwrap_or_else(|e| {
        if is_debug {
            eprintln!("Falha ao ler o arquivo: {}", e);
        }
        kill("002 : FILE_READ_ERROR")
    });

    if content.is_empty() {
        if is_debug {
            println!("O arquivo está vazio.");
        }
        kill("003 : EMPTY_FILE");
    }

    if is_debug {
        println!(
            "Path: {}\nDebug: {}\nLength: {} \n",
            path,
            is_debug,
            content.len()
        );
    }

    let father_path: &Path = if let Some(v) = Path::new(path).parent() {
        v
    } else {
        kill("File Not Found");
    };
    //println!("A\nB\x20C");

    //let mut tree: Environment = Environment::new();

    let mut strings: Vec<String> = Vec::new();

    let mut scopes: Vec<Vec<String>> = Vec::new();

    let lines: Vec<String> = prepare_to_parse(content, is_debug, &mut strings);

    if is_debug {
        let mut c = 0;
        for s in strings {
            println!("*STRING:{} : {}", c, s);
            c += 1;
        }
    }

    let resto: Vec<String> = find_scopes(lines, &mut scopes);

    for r in resto {
        println!("RESTO: {}", r);
    }

    let mut c: usize = 0;
    for s in scopes {
        println!("ESCOPO {} : {:#?}", c, s);
        c += 1;
    }

    return;

    /*
    let global_functions: Vec<String> = vec![];
    let global_values: Vec<String> = vec![];
    */
    let mut values: Vec<Values> = vec![];

    let mut imported_files: Vec<String> = Vec::new();

    if let Some(v) = Path::new(path).file_name().and_then(|n| n.to_str()) {
        if is_debug {
            println!("{}", &v);
        }
        imported_files.push(v.to_string())
    } else {
        kill("File Not Found");
    }

    let _ = import_from_file(lines, &mut values, &is_debug, father_path, &mut strings);
}

#[derive(Debug)]
pub struct Values {
    pub name: String,
    pub data_type: DataTypes,
    pub content: Vec<String>,
    pub origin: String,
    pub public: bool,
}

#[derive(Debug)]
pub enum DataTypes {
    Function,
    Value,
    Type,
}
