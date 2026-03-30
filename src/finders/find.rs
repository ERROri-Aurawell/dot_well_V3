use std::collections::HashMap;

use crate::text_to_vec::structs::FunctionDefinition;

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

pub fn find_scopes(conteudo: Vec<String>, escopos: &mut Vec<Vec<String>>) -> Vec<String> {
    let mut resto: Vec<String> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();

    for mut linha in conteudo {
        if linha.contains("{") {
            linha.truncate(linha.len() - 1);
            let scope_id = escopos.len();
            let separator = if linha.ends_with(' ') || linha.is_empty() {
                ""
            } else {
                " "
            };
            let new_line = format!("{}{}*SCOPE:{}", linha, separator, scope_id);
            if let Some(&current_scope_idx) = stack.last() {
                escopos[current_scope_idx].push(new_line);
            } else {
                resto.push(new_line);
            }

            escopos.push(Vec::new());
            stack.push(scope_id);
        } else if linha.contains("}") {
            linha.truncate(linha.len() - 1);

            if linha.len() > 0 {
                if let Some(&current_scope_idx) = stack.last() {
                    escopos[current_scope_idx].push(linha);
                }
            }

            stack.pop();
        } else {
            if let Some(&current_scope_idx) = stack.last() {
                escopos[current_scope_idx].push(linha);
            } else {
                resto.push(linha);
            }
        }
    }
    resto
}
