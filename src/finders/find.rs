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
                functions: None,
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

fn unassemble_function(
    is_debug: &bool,
    content: &str,
    file: &str,
    type_function: &bool,
    who: &str,
) -> (Option<Vec<Parameter>>, u32, String, String) {
    let mut true_params: Option<Vec<Parameter>> = None;
    let true_name: String;
    let true_return: String;
    let true_id: u32;
    if *is_debug {
        println!("FUNÇÕES: {}", content);
    }

    if let Some((f_name, resto)) = content.replace("+", "").split_once("(") {
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
                    let name: String;
                    let types: String;

                    if let Some((param, tipo)) = p.split_once(":") {
                        name = param.to_string();
                        types = tipo.to_string();
                    } else {
                        if *type_function {
                            name = "Self".to_string();
                            types = who.to_string();
                        } else {
                            kill("FUNCTION PARSING MALFUNCTION");
                        }
                    }

                    the_params.push(Parameter {
                        var_name: name,
                        var_type: types,
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
                let error = format!("FUNCTION INTERNAL ERROR: BUILDING MALFUNCTION: {}", &file);
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

#[derive(Debug, Clone)]
pub struct Functions {
    pub name: String,
    pub params: Option<Vec<Parameter>>,
    pub body_scope_id: u32,
    pub public: bool,
    pub return_type: String,
    pub file: String,
    pub special: Option<String>,
}
#[derive(Debug, Clone)]
pub struct Parameter {
    pub var_name: String,
    pub var_type: String,
}

pub fn return_functions(
    scopes: &mut Vec<Scopes>,
    resto: &mut Vec<Resto>,
    is_debug: &bool,
    type_function: bool,
    who: &str,
    scope_functions: &mut Vec<Functions>,
    global: bool,
) -> Vec<Resto> {
    let mut new_resto: Vec<Resto> = Vec::new();

    for f in resto {
        if !f.content.contains("fn") {
            new_resto.push(Resto {
                file: f.file.clone(),
                content: f.content.clone(),
            });

            continue;
        }

        let content: &String = &f.content.replace(" ", "+");

        let special: Option<String>;
        let is_public: bool;

        let to_extract: String;
        let res: &str;

        if let Some((resto, nome)) = content.split_once("fn+") {
            to_extract = nome.to_string();
            res = resto;
        } else {
            let error = format!("KILLED BY {}", content);
            kill(&error);
        }
        let restos: String;

        if let Some((resto, _)) = res.split_once("public+") {
            is_public = true;
            let ret = resto.replace("+", "");
            restos = ret.clone();

            if !global & !type_function {
                let erro = format!("YOU CANNOT CREATE A PUBLIC FUNCTION IN A LOCAL SCOPE");
                kill(&erro)
            }
        } else {
            is_public = false;
            restos = res.to_string();
        }

        if restos.starts_with("$") {
            special = Some(restos);
        } else {
            special = None;
        }

        //println!("CONTENT: {}", &to_extract);

        let (params, body_scope_id, name, return_type): (
            Option<Vec<Parameter>>,
            u32,
            String,
            String,
        ) = unassemble_function(is_debug, &to_extract, &f.file, &type_function, who);

        for g in &*scope_functions {
            if g.name == name && (g.public && is_public || g.file == f.file) {
                let mut public_g = "";
                let mut public_n = "";

                if g.public {
                    public_g = "public ";
                }

                if is_public {
                    public_n = "public "
                }
                let error = format!(
                    "TWO OR MORE PUBLIC/GLOBAL FUNCTIONS CANNOT HAVE THE SAME NAME. \n| {}{} | & | {}{} |\nFILE: {}",
                    public_g, g.name, public_n, name, f.file
                );
                kill(&error);
            }
        }
        //Chegando aqui é por que nada deu errado anteriormente

        let nf = Functions {
            name,
            params,
            body_scope_id,
            public: is_public,
            return_type,
            file: f.file.clone(),
            special,
        };

        scope_functions.push(nf);

        //Eu preciso da recursividade agora
        let intern_content = &scopes[body_scope_id as usize];
        let mut intern_functions: Vec<Functions> = Vec::new();
        let mut intern_resto: Vec<Resto> = Vec::new();

        for l in &intern_content.lines {
            intern_resto.push(Resto {
                file: intern_content.file.clone(),
                content: l.clone(),
            });
        }
        let mut intern_scope: Scopes = intern_content.clone();

        let scopes_intern_new_lines: Vec<Resto> = return_functions(
            scopes,
            &mut intern_resto,
            is_debug,
            type_function,
            who,
            &mut intern_functions,
            false,
        );

        for f in &intern_functions {
            //println!("\nINTERNAL FUNCTIONS: \n---\n{:#?}\n---", f);
        }

        let mut stringfy: Vec<String> = Vec::new();
        for s in scopes_intern_new_lines {
            stringfy.push(s.content);
        }
        intern_scope.lines = stringfy;

        match &intern_scope.functions {
            None => {
                if !intern_functions.is_empty() {
                    intern_scope.functions = Some(intern_functions);
                }
            }
            Some(s) => {
                if !intern_functions.is_empty() {
                    let mut new_value: Vec<Functions> = s.to_vec();
                    new_value.append(&mut intern_functions);
                    intern_scope.functions = Some(new_value);
                }
            }
        }

        scopes[body_scope_id as usize] = intern_scope;
    }

    new_resto
}

pub fn find_types(
    scopes: &mut Vec<Scopes>,
    resto: &mut Vec<Resto>,
    is_debug: &bool,
) -> (Vec<Resto>, Vec<Type>) {
    let mut raw_types: Vec<RawType> = Vec::new();
    let mut new_resto: Vec<Resto> = Vec::new();

    let mut type_names: Vec<String> = Vec::new();

    for r in resto {
        if !r.content.contains("type") {
            new_resto.push(r.clone());
            continue;
        };
        if *is_debug {
            println!("\nFILTER: {}", &r.content);
        }

        let is_public: bool;
        let resto: &str;
        let true_name: String; //
        let pre_process: &str;
        let paramless: bool;
        let interact: bool;

        if let Some((processor, res)) = r.content.split_once("@") {
            if res.starts_with("ParamLess") {
                pre_process = &res[9..];
                paramless = true;
            } else if res.starts_with("Interact") {
                pre_process = &res[7..];
                paramless = false;
                interact = true;
            } else {
                let error = format!(
                    "INVALID PROCESSOR: |{}|\n|{}|\n|{}|",
                    processor, r.content, r.file
                );
                kill(&error);
            }
        } else {
            pre_process = &r.content;
            paramless = false;
        }

        let replace = pre_process.replace(" ", "+");

        if let Some((_, tipo)) = replace.split_once("public+") {
            is_public = true;
            resto = tipo;
        } else {
            is_public = false;
            resto = &replace;
        }

        if *is_debug {
            println!("RESTO NOVO:{}", &resto);
        }

        /*
        //Método se eu fosse ignorar palavras entre o public e o type
        if let Some((_, tipo)) = resto.split_once("type+") {
            resto = tipo;
        }
        */

        let init: &str = &resto[0..5];

        if *is_debug {
            println!("INIT: {}", &init);
        }

        if init != "type+" {
            let error = format!("TYPE SYNTAX ERROR: \"{}\" : {}", r.content, &r.file);
            kill(&error);
        }

        let resto: String = if paramless {
            let r: &str = &resto[5..];
            format!("{}+", r)
        } else {
            resto[5..].to_string()
        };

        if let Some((nome, id)) = resto.split_once("+") {
            if *is_debug {
                println!("NAME: {}", &nome);
            }
            true_name = nome.to_string();

            let rt: RawType;

            type_names.push(true_name.clone());

            if !paramless {
                rt = extract(id, &r.file, scopes, &*is_debug, &true_name, is_public);
            } else {
                rt = RawType {
                    name: true_name,
                    public: is_public,
                    file: r.file.clone(),
                    fields: None,
                };
            }

            raw_types.push(rt);
        } else {
            let error = format!("TYPE INTERNAL ERROR: BUILDING MALFUNCTION: {}", &r.file);
            kill(&error);
        }
    }
    //

    if *is_debug {
        println!("\nIMPLEMENTANDO FUNÇÕES \n");
    }

    let mut true_resto: Vec<Resto> = Vec::new();

    let mut methods_map: HashMap<String, Vec<Functions>> = HashMap::new();

    for r in new_resto {
        if !r.content.starts_with("impl") {
            true_resto.push(r.clone());
            continue;
        }
        println!("{}\n", r.content);

        let t = &r.content.replace(" ", "+")[5..];
        if let Some((who, id)) = t.split_once("+*SCOPE:") {
            if !type_names.contains(&who.to_string()) {
                let error = format!(
                    "TYPE IMPLEMENTATION ERROR: \"{}\" : {} \nTYPE NOT FOUND",
                    r.content, &r.file
                );
                kill(&error);
            }

            let error = format!(
                "TYPE INTERNAL ERROR: PARSING MALFUNCTION : {} - {}",
                r.content, r.file
            );
            let id: u32 = id.parse().expect(&error);

            let internal_content = &scopes[id as usize];

            println!("INTERNAL CONTENT: {:?}\n", &internal_content);

            let mut inner_resto: Vec<Resto> = Vec::new();

            for c in &internal_content.lines {
                inner_resto.push(Resto {
                    file: internal_content.file.clone(),
                    content: c.to_string(),
                })
            }

            let mut global_functions: Vec<Functions> = Vec::new();

            let _new_resto: Vec<Resto> = return_functions(
                scopes,
                &mut inner_resto,
                is_debug,
                true,
                &who,
                &mut global_functions,
                true,
            );

            if *is_debug {
                for fun in &global_functions {
                    println!("\nFUNÇÕES EXTRAIDAS DO :  {} : {:#?}", &who, &fun);
                }
            }

            for func in global_functions {
                methods_map.entry(who.to_string()).or_default().push(func);
            }
        }
    }

    let final_types: Vec<Type> = raw_types
        .into_iter()
        .map(|raw| {
            let name = raw.name.clone();
            Type {
                type_params: raw,
                methods: methods_map.remove(&name),
            }
        })
        .collect();

    (true_resto, final_types)
}

#[derive(Debug, Clone)]
pub struct Type {
    pub type_params: RawType,
    pub methods: Option<Vec<Functions>>,
}
#[derive(Debug, Clone)]
pub struct RawType {
    pub name: String,
    pub public: bool,
    pub file: String,
    pub fields: Option<Vec<Field>>,
}
#[derive(Debug, Clone)]
pub struct Field {
    pub public: bool,
    pub var_name: String,
    pub var_type: String,
}

fn extract(
    id: &str,
    file: &str,
    scopes: &mut Vec<Scopes>,
    is_debug: &bool,
    true_name: &str,
    public: bool,
) -> RawType {
    let id: &str = &id[7..];
    let error = format!("TYPE INTERNAL ERROR: PARSING MALFUNCTION : {}", file);
    let id: u32 = id.parse().expect(&error);

    let escopo_interno: &Scopes = &scopes[id as usize];
    if *is_debug {
        println!("\nESCOPO INTERNO: {:?}", &escopo_interno);
    }

    let so_close_params: Vec<&str> = escopo_interno.lines[0].split(",").collect();
    if so_close_params.is_empty() {
        let error = format!(
            "TYPE SYNTAX ERROR: NO PARAMS FOUND: \"{}\" : {}",
            true_name, file
        );
        kill(&error);
    }

    let mut params: Vec<Field> = Vec::new();

    for s in so_close_params {
        if let Some((name, type_of)) = s.split_once(":") {
            let true_name: String;
            let public: bool;

            //println!("\nINTERN NAME:{}", &name);

            if let Some((_, name)) = name.replace(" ", "+").split_once("public+") {
                //println!("INTERN NAME_2:{}", &name);

                true_name = name.to_string().replace("+", "");
                public = true
            } else {
                true_name = name.to_string().replace("+", "");
                public = false
            }

            params.push(Field {
                public,
                var_name: true_name,
                var_type: type_of.to_string(),
            });
        } else {
            let error = format!("TYPE SYNTAX ERROR: \"{}\" : {}", true_name, &file);
            kill(&error);
        }
    }

    RawType {
        name: true_name.to_string(),
        public,
        file: file.to_string(),
        fields: Some(params),
    }
}
