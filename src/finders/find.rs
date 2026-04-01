use std::collections::HashMap;

use crate::text_to_vec::structs::FunctionDefinition;

use std::path::{Path, PathBuf};

use crate::kill;

use crate::Scopes;

pub fn find_func(conteudo: Vec<String>) {
    // Funções/Resto (Vec<FunctionDefinition>, Vec<String>)
    let mut resto: Vec<String> = Vec::new();

    let mut func_form: bool = false;

    for linha in conteudo {
        //println!("{}", n);

        if linha.contains("fn") || func_form {
            println!("LINHA: {}", linha)
        } else {
            resto.push(linha)
        }
    }
}

pub fn find_scopes(conteudo: Vec<String>, escopos: &mut Vec<Scopes>, resto: &mut Vec<String>, file: &String) {
    let mut profundidade: usize = 0;
    let mut stack: Vec<usize> = Vec::new();
    for mut linha in conteudo {
        if linha.contains("{") {
            //println!("PROFUNDIDADE: {}\nLINHA : {}", profundidade, &linha);

            linha.truncate(linha.len() - 1);
            let scope_id = escopos.len();
            let separator = if linha.ends_with(' ') || linha.is_empty() {
                ""
            } else {
                " "
            };
            let new_line = format!("{}{}*SCOPE:{}", linha, separator, scope_id);
            if let Some(&current_scope_idx) = stack.last() {
                escopos[current_scope_idx].lines.push(new_line);
            } else {
                resto.push(new_line);
            }

            let scope = Scopes {
                depth: profundidade as u32,
                lines: Vec::new(),
                file: file.to_string()
            };

            escopos.push(scope);
            stack.push(scope_id);

            profundidade = profundidade + 1;
        } else if linha.contains("}") {
            linha.truncate(linha.len() - 1);

            profundidade = profundidade - 1;

            if linha.len() > 0 {
                if let Some(&current_scope_idx) = stack.last() {
                    escopos[current_scope_idx].lines.push(linha);
                }
            }

            stack.pop();
        } else {
            if let Some(&current_scope_idx) = stack.last() {
                escopos[current_scope_idx].lines.push(linha);
            } else {
                resto.push(linha);
            }
        }
    }
}

pub fn find_imports(
    linhas: Vec<String>,
    master: &bool,
    path: &Path,
) -> (Vec<PathBuf>, Vec<String>) {
    let mut files_to_import: Vec<PathBuf> = Vec::new();
    let mut novo_resto: Vec<String> = Vec::new();

    for linha in linhas {
        if !linha.starts_with("import") {
            novo_resto.push(linha.clone());
            continue;
        };

        if !master {
            kill("ONLY MASTER CAN INPORT FILES");
        }

        let mut chars = linha.chars();
        chars.next_back();
        let path: PathBuf = path.join(&chars.as_str()[7..]);

        files_to_import.push(path);
    }

    (files_to_import, novo_resto)
}

pub fn find_public(scopes: &mut Vec<Vec<String>>, global: &mut Vec<String>, is_debug: &bool) {
    for escopo in scopes {
        for linha in escopo {
            if linha.starts_with("public") {
                for g in &*global {}
            }
        }
    }
}
