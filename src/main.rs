pub mod finders;
mod stages;
mod text_to_vec;
mod vec_to_byte;

use std::path::Path;
use std::{env, fs, process};

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

    let (is_debug, path) = match args.get(1).map(|s: &String| s.as_str()) {
        Some("--debug") => (
            true,
            args.get(2)
                .unwrap_or_else(|| kill("001 : NO_FILE_PROVIDED")),
        ),
        Some(p) => (false, &p.to_string()),
        None => kill("001 : NO_FILE_PROVIDED"),
    };

    let content: String = fs::read_to_string(&path).unwrap_or_else(|e: std::io::Error| {
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

    let mut is_master: bool = true;
    let mut strings: Vec<String> = Vec::new();
    let mut scopes: Vec<Scopes> = Vec::new();
    let mut resto: Vec<Resto> = Vec::new();
    let mut imported_files: Vec<String> = Vec::new();

    stages::one::first_one(
        content,
        &father_path,
        &path,
        &is_debug,
        &mut imported_files,
        &mut strings,
        &mut scopes,
        &mut resto,
        &mut is_master,
    );

    if is_debug {
        println!("Importações concluidas... eu acho");
    }
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

pub struct Scopes{
    pub depth: u32,
    pub lines: Vec<String>,
    pub file: String,
}

pub struct Resto{
    file: String,
    content: String
}