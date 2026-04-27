use crate::text_to_vec::prepare_terrain::prepare_to_parse;

use crate::finders::find::{
    Functions, Type, find_imports, find_scopes, find_types, return_functions,
};

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
    imported_files: &mut Vec<String>,
    strings: &mut Vec<String>,
    scopes: &mut Vec<Scopes>,
    novo_resto: &mut Vec<Resto>,
    is_master: &mut bool,
    father_id: &mut Vec<usize>,
) -> Option<(Vec<Type>, Vec<Functions>)> {
    // Estágio 1.1: Limpeza e Substituição de Strings.
    let lines: Vec<String> = prepare_to_parse(content, *is_debug, strings);

    // Estágio 1.2: Identificação de Escopos (Substitui blocos por tokens SCOPE:X).
    let mut resto: Vec<Resto> = Vec::new();
    find_scopes(lines, scopes, &mut resto, &path, father_id);

    if *is_debug {
        println!("\n--- TABELA DE STRINGS ---");
        for (c, s) in strings.iter().enumerate() {
            println!("[{:3}] -> \"{}\"", c, s);
        }

        println!("\n--- ESCOPOS IDENTIFICADOS ---");
        for (c, s) in scopes.iter().enumerate() {
            println!("ID: {:3} | Arquivo: {}", c, s.file);
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
        return None;
    }

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
                imported_files,
                strings,
                scopes,
                novo_resto,
                is_master,
                father_id,
            );
        }
    }

    println!("EU SÓ SOU CHAMADO UMA VEZ, DEPOIS DE TODOS\n\n");

    // Estágio 1.5: Públicos válidos e funções

    if *is_debug {
        let mut contador: usize = 0;
        for global in &*scopes {
            contador = contador + 1;
            println!("SCOPE {}: {:#?}", contador - 1, global);
        }

        println!("\n");
    }

    let mut global_functions: Vec<Functions> = Vec::new();
    let new_resto: Vec<Resto> = return_functions(
        scopes,
        novo_resto,
        is_debug,
        false,
        "",
        &mut global_functions,
        true,
    );

    *novo_resto = new_resto;

    if *is_debug {
        for global in &*scopes {
            if global.functions.is_some() {
                println!("ESCOPOS COM FUNÇÃO: {:#?}", global);
            }
        }

        println!("\n");

        for f_global in &global_functions {
            println!("FUNÇÕES GLOBAIS: {:#?}", f_global);
        }

        println!("\n---\n");

        println!("COMEÇANDO PROCESSAMENTO DOS TYPES: ");
    }

    // 1.6 Tipos e Extensões
    let (resto, types) = find_types(scopes, novo_resto, &is_debug, true);

    *novo_resto = resto;

    if *is_debug {
        for f in &types {
            println!("TYPES: {:#?}", f);
        }

        println!("\n\nIMPLEMENTANDO TIPOS POR ESCOPO\n");
    }

    for (id, esc) in scopes.clone().iter().enumerate() {
        let mut esc_como_resto: Vec<Resto> = Vec::new();

        for l in esc.lines.clone() {
            esc_como_resto.push(Resto {
                file: esc.file.clone(),
                content: l.clone(),
            })
        }

        let (conteudo, tipos) = find_types(scopes, &mut esc_como_resto, &is_debug, false);

        let mut resto_como_conteudo: Vec<String> = Vec::new();

        for c in conteudo {
            resto_como_conteudo.push(c.content);
        }

        scopes[id] = Scopes {
            lines: resto_como_conteudo,
            file: esc.file.clone(),
            father_id: esc.father_id,
            functions: esc.functions.clone(),
            types: { if tipos.len() > 0 { Some(tipos) } else { None } },
        };
    }
    Some((types, global_functions))
}
