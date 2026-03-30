use crate::{DataTypes, Values, kill};
use std::path::Path;
use std::{env, fs, process};

use crate::finders::find::find_func;

use crate::text_to_vec::prepare_terrain::prepare_to_parse;

pub fn import_from_file(
    import: Vec<String>,
    valores: &mut Vec<Values>,
    is_debug: &bool,
    path: &Path,
    strings: &mut Vec<String>
) {
    for v in import {
        if v.starts_with("import") {
            let mut chars = v.chars();
            chars.next_back();
            let path = path.join(&chars.as_str()[7..]);

            if let Some(v) = Path::new(&path).file_name().and_then(|n| n.to_str()) {
                //println!("{}, {:?}", v, path);
                let content = fs::read_to_string(&path).unwrap_or_else(|e| {
                    if *is_debug {
                        eprintln!("Falha ao ler o arquivo: {}", e);
                    }
                    kill("002 : IMPORT_FILE_READ_ERROR")
                });

                if content.is_empty() {
                    if *is_debug {
                        println!("O arquivo está vazio.");
                    }
                    //("003 : EMPTY_FILE");
                    continue;
                }

                if *is_debug {
                    println!("\nLendo conteudo de : {}\n", &path.display());
                }

                let lines: Vec<String> = prepare_to_parse(content, *is_debug, strings);

                    let _ = find_func(lines);
                    
            } else {
                kill("File Not Found");
            }
        }
    }
}

/*

fn what_is_this(line: Vec<String>, origin: &str, valores: &mut Vec<Values>) {
    let mut in_function: bool = false;
    let mut content: Vec<String> = vec![];
    let mut name: String = "".to_string();
    let mut public: bool = false;
    let mut data_type: DataTypes = DataTypes::Function;

    for l in line {
        if in_function {
            if l.ends_with("}") {
                in_function = false;
                content.push(l);

                let new_value = Values {
                    name: name.clone(),
                    data_type,
                    content,
                    origin: origin.to_string(),
                    public,
                };

                data_type = DataTypes::Function;

                valores.push(new_value);

                content = Vec::new();
                public = false;
                continue;
            }
            content.push(l);
            continue;
        }

        let (f, s) = match l.split_once(" ") {
            Some(f) => f,
            None => ("", ""),
        };

        match f {
            "public" => {
                public = true;
                anything_else(
                    s,
                    &mut in_function,
                    &mut name,
                    &mut data_type,
                    &mut content,
                    valores,
                    &origin,
                    &mut public,
                );
            }
            _ => {
                anything_else(
                    f,
                    &mut in_function,
                    &mut name,
                    &mut data_type,
                    &mut content,
                    valores,
                    &origin,
                    &mut public,
                );
            }
        }
    }

    for v in valores {
        println!("{:?}", v);
    }
}

fn anything_else(
    resto: &str,
    in_function: &mut bool,
    name: &mut String,
    data_type: &mut DataTypes,
    content: &mut Vec<String>,
    valores: &mut Vec<Values>,
    origin: &str,
    public: &mut bool,
) {
    let (f, s) = match resto.split_once(" ") {
        Some(f) => f,
        None => ("", ""),
    };
    match f {
        "fn" => {
            println!("É uma função sendo definida");
            println!("{}", &s);
            println!("{}", &resto);
            let (nome, resto) = match s.split_once("(") {
                Some(f) => f,
                None => ("", ""),
            };

            *name = nome.to_string();
            content.push("(".to_string() + resto);

            *in_function = true;
            *data_type = DataTypes::Function;
        }
        "type" => {
            println!("É um tipo sendo definido");
            let (nome, resto) = match s.split_once("{") {
                Some(f) => f,
                None => ("", ""),
            };

            *name = nome.to_string();
            content.push("{".to_string() + resto);

            *in_function = true;
            *data_type = DataTypes::Type;
        }
        "let" => {
            println!("É uma variavel sendo definida");
            let (nome, resto) = match s.split_once(" ") {
                Some(f) => f,
                None => ("", ""),
            };

            *name = nome.to_string();
            content.push(resto.to_string());

            let new_value = Values {
                name: name.clone(),
                data_type: DataTypes::Value,
                content: content.to_vec(),
                origin: origin.to_string(),
                public: *public,
            };

            valores.push(new_value);

            *content = Vec::new();
            *public = false;
        }
        _ => {}
    }
}

*/
