use crate::text_to_vec::prepare_terrain::prepare_to_parse;

use crate::finders::find::{find_func, find_imports, find_scopes};

use std::fs;

use crate::{Resto, Scopes, kill};

use std::path::{Path, PathBuf};

pub fn first_one(
    content: String,
    father_path: &Path,
    path: &String,
    is_debug: &bool,
    imported_files: &mut Vec<String>,
    strings: &mut Vec<String>,
    scopes: &mut Vec<Scopes>,
    novo_resto: &mut Vec<Resto>,
    is_master: &mut bool,
) {
    let mut resto: Vec<Resto> = Vec::new();

    //Substitui todas as strings por tokens de STRING:X, onde X é o índice do array "strings".
    let lines: Vec<String> = prepare_to_parse(content, *is_debug, strings);

    //Substitui todos os escopos por tokens de SCOPE:X, onde X é o índice do array "scopes".
    find_scopes(lines, scopes, &mut resto, &path);

    if *is_debug {
        for (c, s) in strings.iter().enumerate() {
            println!("*STRING:{} : {}", c, s);
        }

        for (c, s) in scopes.iter().enumerate() {
            println!(
                "ESCOPO {}\nPROFUNDIDADE DO ESCOPO: {}\nARQUIVO:{}\n {:#?}",
                c, s.depth, s.file, s.lines
            );
        }
    }

    //puxa do resto o caminho dos arquivos a importar. Importações DEVEM estar no escopo global.
    let (files_to_import, mut resto) = find_imports(resto, &is_master, &father_path);

    //Apenas o Master deve importar arquivos.
    *is_master = false;

    //Une o global de todos os arquivos.
    novo_resto.append(&mut resto);

    if *is_debug {
        if *is_master {
            for r in &files_to_import {
                println!("ARQUIVOS A IMPORTAR: {:#?}", r);
            }
        }

        for r in &*novo_resto {
            println!("NOVO RESTO: {}\nDO ARQUIVO: {}\n", r.content, r.file);
        }
    }

    //Puxa o nome e termina a importação.
    if let Some(v) = Path::new(path).file_name().and_then(|n| n.to_str()) {
        if *is_debug {
            println!("\n{} IMPORTADO COM SUCESSO\n", &v);
        }
        imported_files.push(v.to_string())
    } else {
        kill("File Not Found");
    }

    //Começa a importação dos outros arquivos
    for file_to_import in files_to_import {
        if *is_debug {
            println!("Importando : {:?}\n", &file_to_import);
        }
        let content: String =
            fs::read_to_string(&file_to_import).unwrap_or_else(|e: std::io::Error| {
                if *is_debug {
                    eprintln!("Falha ao ler o arquivo: {}", e);
                }
                kill("002 : FILE_READ_ERROR")
            });

        if let Some(path) = file_to_import.to_str() {
            let path = path.to_string();

            if content.is_empty() {
                if *is_debug {
                    println!("O arquivo está vazio.");
                }
                kill("003 : EMPTY_FILE");
            }

            if *is_debug {
                println!(
                    "Path: {:?}\nDebug: {}\nLength: {} \n",
                    path,
                    is_debug,
                    content.len()
                );
            }

            first_one(
                content,
                father_path,
                &path,
                is_debug,
                imported_files,
                strings,
                scopes,
                novo_resto,
                is_master,
            );
        }
    }
}
