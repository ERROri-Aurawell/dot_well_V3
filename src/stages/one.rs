use crate::text_to_vec::prepare_terrain::prepare_to_parse;

use crate::finders::find::{find_function, find_imports, find_scopes, find_types};

use std::fs;

use crate::{Resto, Scopes, kill};

use std::path::{Path, PathBuf};

/// Primeiro estágio do compilador/transpilador.
///
/// Este estágio realiza as seguintes tarefas:
/// 1. Limpeza e normalização do código (removendo comentários e espaços extras).
/// 2. Tokenização de strings e identificação de escopos (blocos `{}`).
/// 3. Resolução recursiva de arquivos importados.
pub fn first_one(
    content: String,
    father_path: &Path,
    path: &String,
    is_debug: &bool,
    first: &bool,
    imported_files: &mut Vec<String>,
    strings: &mut Vec<String>,
    scopes: &mut Vec<Scopes>,
    novo_resto: &mut Vec<Resto>,
    is_master: &mut bool,
) {
    let mut resto: Vec<Resto> = Vec::new();

    // Estágio 1.1: Limpeza e Substituição de Strings.
    let lines: Vec<String> = prepare_to_parse(content, *is_debug, strings);

    // Estágio 1.2: Identificação de Escopos (Substitui blocos por tokens SCOPE:X).
    find_scopes(lines, scopes, &mut resto, &path);

    if *is_debug {
        println!("\n--- TABELA DE STRINGS ---");
        for (c, s) in strings.iter().enumerate() {
            println!("[{:3}] -> \"{}\"", c, s);
        }

        println!("\n--- ESCOPOS IDENTIFICADOS ---");
        for (c, s) in scopes.iter().enumerate() {
            println!(
                "ID: {:3} | Profundidade: {} | Arquivo: {}",
                c, s.depth, s.file
            );
            for line in &s.lines {
                println!("  | {}", line);
            }
        }
    }

    // Estágio 1.3: Extração de Importações (Devem estar no escopo global).
    let (files_to_import, mut resto) = find_imports(resto, &is_master, &father_path);

    // Acumula o conteúdo global processado (o que não está dentro de escopos).
    novo_resto.append(&mut resto);

    if *is_debug {
        if !files_to_import.is_empty() {
            println!("\n--- ARQUIVOS PENDENTES PARA IMPORTAÇÃO ---");
            for r in &files_to_import {
                println!(" -> {:?}", r);
            }
        }

        println!("\n--- CONTEÚDO GLOBAL (RESTO) ---");
        for r in &*novo_resto {
            println!("[{}] {}", r.file, r.content);
        }
    }

    // Registra o arquivo atual na lista de arquivos já processados.
    if let Some(v) = Path::new(path).file_name().and_then(|n| n.to_str()) {
        if *is_debug {
            println!("\n[OK] '{}' carregado com sucesso.", &v);
        }
        imported_files.push(v.to_string())
    } else {
        kill("File Not Found");
    }

    if !*is_master {
        return;
    }

    //println!("QUANTAS VEZES EU VOU SER CHAMADO?????\n------\n------\n-------\n\n");
    // Apenas o arquivo Master (raiz) pode iniciar o processo de importação em cascata.
    *is_master = false;

    // Estágio 1.4: Recursividade (Processa cada arquivo encontrado no comando 'import').
    for file_to_import in files_to_import {
        if *is_debug {
            println!("\n[IMPORT] Lendo dependência: {:?}", &file_to_import);
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
                println!(" -> Path: {}\n -> Size: {} bytes", path, content.len());
            }

            first_one(
                content,
                father_path,
                &path,
                is_debug,
                first,
                imported_files,
                strings,
                scopes,
                novo_resto,
                is_master,
            );
        }
    }

    //println!("EU SÓ SOU CHAMADO UMA VEZ, DEPOIS DE TODOS\n\n");

    // Estágio 1.5: Públicos válidos e funções
    let (funcoes_globais, funcoes_locais, new_r, _) = find_function(scopes, novo_resto, is_debug, false, "");

    *novo_resto = new_r;

    let mut files: Vec<String> = Vec::new();

    for f in &funcoes_locais {
        if *is_debug {
            println!(
                "Função: {} | \nPai: Escopo {}\n",
                &f.function.name, &f.father
            );
        }

        let scope: &Scopes = &scopes[f.father as usize];

        if files.contains(&scope.file) {
            continue;
        }

        let mut new_lines: Vec<String> = Vec::new();

        for l in scope.lines.iter() {
            if *is_debug {
                println!("L - PAI: {}", &l);
            }

            if l.starts_with("fn") {
                continue;
            }
            new_lines.push(l.to_string());
        }

        let new_scope = Scopes {
            depth: scope.depth,
            lines: new_lines,
            file: scope.file.clone(),
        };

        files.push(scope.file.clone());

        scopes[f.father as usize] = new_scope;
    }

    if *is_debug {
        println!("\n--- CONTEÚDO GLOBAL (RESTO) ---");
        for r in &*novo_resto {
            println!("[{}] {}", r.file, r.content);
        }

        println!("\n--- ESCOPOS IDENTIFICADOS ---");
        for (c, s) in scopes.iter().enumerate() {
            println!(
                "ID: {:3} | Profundidade: {} | Arquivo: {}",
                c, s.depth, s.file
            );
            for line in &s.lines {
                println!("  | {}", line);
            }
        }
    }

    // 1.6 Tipos e Extensões
    find_types(scopes, novo_resto, &is_debug);
}
