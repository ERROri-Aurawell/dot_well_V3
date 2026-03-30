use crate::{DataTypes, Values, kill};
use std::path::Path;
use std::{env, fs, process};

use crate::prepare_to_parse;

pub fn import_from_file(
    import: Vec<String>,
    valores: &mut Vec<Values>,
    is_debug: &bool,
    path: &Path,
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
                    kill("002 : FILE_READ_ERROR")
                });

                if content.is_empty() {
                    if *is_debug {
                        println!("O arquivo está vazio.");
                    }
                    //("003 : EMPTY_FILE");
                    continue;
                }

                let lines: Vec<String> = prepare_to_parse(content, *is_debug);

                what_is_this(lines, &v, valores);
            } else {
                kill("File Not Found");
            }
        }
    }
}

fn what_is_this(line: Vec<String>, origin: &str, valores: &mut Vec<Values>) {
    let mut in_function: bool = false;
    let mut content: Vec<String> = vec![];
    let mut name: String = "".to_string();
    let mut public: bool = false;
    let mut data_type: DataTypes;

    for l in line {
        if in_function {
            if &l == "}" {
                in_function = false;
                content.push(l);

                let new_value = Values {
                    name: name.clone(),
                    data_type,
                    content,
                    origin: origin.to_string(),
                    public,
                };

                valores.push(new_value);

                content = Vec::new();
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
                anything_else(s);
            }
            _ => {
                anything_else(f);
            }
        }
    }
}

fn anything_else(resto: &str) {}
