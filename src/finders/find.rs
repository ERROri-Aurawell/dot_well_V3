use std::collections::HashMap;

use std::path::{Path, PathBuf};

use crate::{Resto, Scopes, kill};

/// Identifica e extrai blocos de código delimitados por chaves `{}`.
///
/// Substitui o conteúdo do escopo por um token `*SCOPE:ID` no fluxo principal ou no escopo pai.
pub fn find_scopes(
    conteudo: Vec<String>,
    escopos: &mut Vec<Scopes>,
    resto: &mut Vec<Resto>,
    file: &String,
) {
    let mut profundidade: usize = 0;
    let mut stack: Vec<usize> = Vec::new();
    for mut linha in conteudo {
        if linha.contains("{") {
            // Início de um novo escopo.
            linha.truncate(linha.len() - 1);
            let scope_id = escopos.len();
            let separator = if linha.ends_with(' ') || linha.is_empty() {
                ""
            } else {
                " "
            };
            // Cria o token que aponta para este novo escopo.
            let new_line = format!("{}{}*SCOPE:{}", linha, separator, scope_id);
            if let Some(&current_scope_idx) = stack.last() {
                escopos[current_scope_idx].lines.push(new_line);
            } else {
                let r = Resto {
                    file: file.clone(),
                    content: new_line,
                };
                resto.push(r);
            }

            let scope = Scopes {
                depth: profundidade as u32,
                lines: Vec::new(),
                file: file.to_string(),
            };

            escopos.push(scope);
            stack.push(scope_id);

            profundidade = profundidade + 1;
        } else if linha.contains("}") {
            // Fim de um escopo.
            linha.truncate(linha.len() - 1);

            profundidade = profundidade - 1;

            if linha.len() > 0 {
                if let Some(&current_scope_idx) = stack.last() {
                    escopos[current_scope_idx].lines.push(linha);
                }
            }

            stack.pop();
        } else {
            // Linha comum: adiciona ao escopo atual ou ao global (resto).
            if let Some(&current_scope_idx) = stack.last() {
                escopos[current_scope_idx].lines.push(linha);
            } else {
                let r = Resto {
                    file: file.clone(),
                    content: linha,
                };
                resto.push(r);
            }
        }
    }
}

/// Analisa as linhas globais em busca de comandos `import`.
///
/// Retorna uma lista de caminhos de arquivos a serem lidos e o conteúdo que sobra (sem os imports).
pub fn find_imports(linhas: Vec<Resto>, master: &bool, path: &Path) -> (Vec<PathBuf>, Vec<Resto>) {
    let mut files_to_import: Vec<PathBuf> = Vec::new();
    let mut novo_resto: Vec<Resto> = Vec::new();

    for linha in linhas {
        if !linha.content.starts_with("import") {
            // Se não for import, mantém no conteúdo global.
            let resto = Resto {
                file: linha.file,
                content: linha.content,
            };
            novo_resto.push(resto);
            continue;
        };

        // Validação de segurança: apenas o arquivo principal pode importar outros.
        if !master {
            kill("ONLY MASTER CAN INPORT FILES");
        }

        let mut chars = linha.content.chars();
        chars.next_back();
        let path: PathBuf = path.join(&chars.as_str()[7..]);

        files_to_import.push(path);
    }

    (files_to_import, novo_resto)
}

pub fn find_function(
    scopes: &mut Vec<Scopes>,
    resto: &mut Vec<Resto>,
    is_debug: &bool,
) -> (Vec<Function>, Vec<LocalFunction>, Vec<Resto>) {
    let mut global: Vec<Function> = Vec::new(); // Funções globais (do arquivo/projeto)
    let mut local: Vec<LocalFunction> = Vec::new(); //Funções locais
    let mut new_resto: Vec<Resto> = Vec::new(); //O restante da global.

    analyze_fn(&is_debug, resto, &mut global, &mut new_resto);

    for f in &global {
        analyze_local_fn(
            is_debug,
            &scopes[f.body_scope_id as usize],
            f.body_scope_id,
            &scopes,
            &mut local,
        );
    }
    if *is_debug {
        println!("\n\n-------------");
        for f in &global {
            println!("\nFUNÇÃO: {:?}", f);
        }

        for f in &local {
            println!("\nFUNÇÃO LOCAL: {:?}", f);
        }
    }

    (global, local, new_resto)
}

#[derive(Debug)]
pub struct Parameter {
    pub var_name: String,
    pub var_type: String,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub params: Option<Vec<Parameter>>,
    pub body_scope_id: u32,
    pub public: bool,
    pub return_type: String,
}

#[derive(Debug)]
pub struct LocalFunction {
    pub function: Function,
    pub father: u32,
}

fn analyze_fn(
    is_debug: &bool,
    resto: &mut Vec<Resto>,
    global: &mut Vec<Function>,
    new_resto: &mut Vec<Resto>,
) {
    for f in resto {
        let content: &str;
        let public: bool;
        if f.content.starts_with("public") {
            content = &f.content[7..];
            public = true;
        } else {
            content = &f.content;
            public = false;
        }

        if !content.starts_with("fn") {
            new_resto.push(Resto {
                file: f.file.clone(),
                content: f.content.clone(),
            });

            continue;
        }
        let content = &content[3..];

        let (params, body_scope_id, name, return_type): (
            Option<Vec<Parameter>>,
            u32,
            String,
            String,
        ) = unassemble_function(is_debug, content, &f.file);

        let nf = Function {
            name,
            params,
            body_scope_id,
            public,
            return_type,
        };
        global.push(nf);
    }
}

fn analyze_local_fn(
    is_debug: &bool,
    scope: &Scopes,
    father_id: u32,
    scopes: &Vec<Scopes>,
    local: &mut Vec<LocalFunction>,
) {
    println!("CORPO DO ESCOPO: {} |\n {:?}", father_id, &scope);
    for line in &*scope.lines {
        //println!("\nLINE: {}", &l);
        if line.starts_with("public") {
            let msg = format!(
                "A INTERNAL SCOPE CANNOT HAVE A PUBLIC DECLARATION | FILE: {} ",
                &scope.file
            );
            kill(&msg);
        };

        if line.contains("*SCOPE:") {
            println!("\nLINHA DE ALGUM ESCOPO: {}", &line);

            if let Some((mut resto, id)) = line.split_once("*SCOPE:") {
                let id_to_check: u32 = id
                    .parse()
                    .expect("FUNCTION INTERNAL ERROR: PARSING MALFUNCTION");

                analyze_local_fn(
                    is_debug,
                    &scopes[id_to_check as usize],
                    id_to_check,
                    scopes,
                    local,
                );

                println!("RESTO DA LINHA COM ESCOPO: {}", &resto);

                if resto.starts_with("fn") {
                    resto = &resto[3..];

                    let (params, body_scope_id, name, return_type): (
                        Option<Vec<Parameter>>,
                        u32,
                        String,
                        String,
                    ) = unassemble_function(
                        is_debug,
                        &format!("{} *SCOPE:{}", resto, id),
                        &scope.file,
                    );

                    println!(
                        "{:?} {} {} {}",
                        &params, &body_scope_id, &name, &return_type
                    );

                    let nf = Function {
                        name,
                        params,
                        body_scope_id,
                        public: false,
                        return_type,
                    };

                    let nf = LocalFunction {
                        function: nf,
                        father: father_id,
                    };

                    local.push(nf);
                }
            }
        }
    }
}

fn unassemble_function(
    is_debug: &bool,
    content: &str,
    file: &str,
) -> (Option<Vec<Parameter>>, u32, String, String) {
    let mut true_params: Option<Vec<Parameter>> = None;
    let true_name: String;
    let true_return: String;
    let true_id: u32;
    if *is_debug {
        println!("FUNÇÕES: {}", content);
    }

    if let Some((f_name, resto)) = content.replace(" ", "").split_once("(") {
        println!("NOME: {}", &f_name);
        true_name = f_name.to_string();

        if let Some((params, resto)) = resto.split_once(")") {
            if params.is_empty() {
                println!("FUNÇÂO SEM PARAMETROS");
            } else {
                let params: Vec<&str> = params.split(",").collect();
                let mut the_params: Vec<Parameter> = Vec::new();

                println!("PARAMETROS: {:?}", &params);

                for p in params {
                    let name_type: Vec<&str> = p.split(":").collect();
                    the_params.push(Parameter {
                        var_name: name_type[0].to_string(),
                        var_type: name_type[1].to_string(),
                    });
                }

                true_params = Some(the_params);
            }

            println!("RESTO:-{}", resto);
            if let Some((return_type, scope_id)) = resto.split_once("*") {
                println!("RETURN TYPE: {}\n", &return_type);

                if return_type.is_empty() {
                    true_return = "Null".to_string();
                } else {
                    true_return = return_type[2..].to_string();
                };

                let id: &str = &scope_id[6..];

                let error = format!("FUNCTION INTERNAL ERROR: PARSING MALFUNCTION : {}", &file);
                true_id = id.parse().expect(&error);
            } else {
                let error = format!("FUNCTION INTERNAL ERROR: PARSING MALFUNCTION: {}", &file);
                kill(&error);
            }
        } else {
            let error = format!("FUNCTION SYNTAX ERROR: NO ENDING FOUND: {}", &file);
            kill(&error);
        }
    } else {
        let error = format!("FUNCTION SYNTAX ERROR: NO NAME FOUND: {}", &file);
        kill(&error);
    };

    (true_params, true_id, true_name, true_return)
}
