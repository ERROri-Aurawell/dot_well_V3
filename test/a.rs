use crate::kill;
use crate::parsing_functions::structs::Function;

pub fn prepare_to_parse(content: &str, is_debug: bool) -> (Vec<String>, Vec<Function>) {
    let mut lines: Vec<String> = Vec::new();

    let mut dentro_string = false;
    let mut fechamento_aguardado: char = '"';

    let pre_result: Vec<String> = content
        .lines()
        .filter(|linha| !linha.trim().is_empty())
        .map(|linha| processar_linha(linha, &mut dentro_string, &mut fechamento_aguardado))
        .collect();

    let mut can_continue = true;

    for linhe in pre_result.iter() {
        if !linhe.trim().is_empty() {
            if linhe.ends_with(";") {
                //println!("testando linha: {}", linhe);

                if can_continue {
                    lines.push(linhe.to_string());
                } else {
                    let position = lines.len() - 1;
                    lines[position].push_str(&linhe);
                    can_continue = true;
                }
            } else {
                can_continue = false;
                lines.push(linhe.to_string());
            }
        }
    }

    for i in lines.iter() {
        if is_debug {
            println!("Linha preparada: {}", i);
        }
    }

    extract_functions(&mut lines, is_debug)
}

fn processar_linha(
    linha: &str,
    dentro_string: &mut bool,
    fechamento_aguardado: &mut char,
) -> String {
    //println!("Processando linha: {}", linha);
    let mut resultado = String::new();
    let mut chars = linha.chars().peekable();

    let mut is_comment = 0; //Se o caractere "\" for encontrado, será adicionado +1. caso valor atinja 2, será considerado comentário. Qualquer coisa diferente de "\" zera o contador.

    while let Some(c) = chars.next() {
        if c != '/' {
            is_comment = 0;
        }

        if *dentro_string {
            //println!("Dentro de string: {}", c);

            resultado.push(c);
            if c == *fechamento_aguardado {
                *dentro_string = false;
            }
            continue;
        }

        if c == '"' || c == '\'' {
            //println!("Iniciando string com: {}", c);

            *dentro_string = true;
            *fechamento_aguardado = c;
            resultado.push(c);
            continue;
        }

        if c == '/' {
            is_comment += 1;
            if is_comment == 2 {
                resultado.pop();
                break;
            }
            resultado.push(c);
            continue;
        }

        if c == ' ' || c == '\t' {
            continue;
        }

        resultado.push(c);
    }

    resultado
}

fn extract_functions(lines: &mut Vec<String>, is_debug: bool) -> (Vec<String>, Vec<Function>) {
    let mut functions: Vec<Function> = Vec::new();

    let mut code_widouth_functions: Vec<String> = Vec::new();

    let mut current_function = Function {
        name: String::new(),
        parameters: Vec::new(),
        code: Vec::new(),
    };

    let mut in_function = false;

    let mut steps_to_end = 0; //Cada abertura de chaves "{" adiciona +1, cada fechamento "}" subtrai -1. Quando chegar a 0, a função terminou.

    for line in lines.iter() {
        if is_debug {
            println!("Analisando linha para funções: '{}'", line);
        };

        if !in_function {
            let (metodo, resto) = match line.split_once(":") {
                Some(v) => v,
                None => ("", line.as_str()),
            };

            if is_debug {
                println!("Método: '{}'\nResto: '{}'", metodo, resto);
            };

            if metodo != "FN" {
                code_widouth_functions.push(line.to_string());
                continue;
            }

            if metodo == "FN" {
                in_function = true;

                if is_debug {
                    println!("Iniciando extração de função.");
                };

                let (name, resto) = match resto.split_once("(") {
                    Some(v) => v,
                    None => kill("010 : FUNCTION_START_PARAMS_MISSING"),
                };
                current_function.name = name.to_string();

                if is_debug {
                    println!("Nome da função: '{}'", current_function.name);
                };

                let (params, resto) = match resto.split_once(")") {
                    Some(v) => v,
                    None => kill("011 : FUNCTION_STOP_PARAMS_MISSING"),
                };

                if is_debug {
                    println!("Parâmetros da função: '{}'", params);
                };

                if !params.is_empty() {
                    let param_list: Vec<&str> = params.split(",").collect();
                    for p in param_list {
                        let param = p.trim();
                        current_function.parameters.push(param.to_string());
                    }
                }

                let (_, function_code) = match resto.split_once("{") {
                    Some(v) => v,
                    None => kill("012 : FUNCTION_CODE_MISSING"),
                };

                if is_debug {
                    println!("Início do código da função: '{}'", function_code);
                };

                steps_to_end += 1; //Conta a primeira chave de abertura da função

                if function_code.contains("{") {
                    steps_to_end += function_code.matches("{").count();
                } // Conta chaves de abertura adicionais na mesma linha

                match function_code.split_once("}") {
                    // Verifica se a função termina na mesma linha
                    Some((code, resto)) => {
                        steps_to_end -= 1; // Conta a chave de fechamento encontrada

                        if steps_to_end == 0 {
                            in_function = false;

                            if is_debug {
                                println!("Função termina na mesma linha.");
                                println!("Código da função: '{}'", code);
                            };

                            let c_lines = code.split(";").collect::<Vec<&str>>();

                            for cl in c_lines {
                                current_function.code.push(format!("{cl};"));
                            }

                            functions.push(current_function);

                            current_function = Function {
                                name: String::new(),
                                parameters: Vec::new(),
                                code: Vec::new(),
                            };

                            let c_lines = resto.split(";").collect::<Vec<&str>>();

                            for cl in c_lines {
                                code_widouth_functions.push(format!("{cl};"));
                            }

                            continue;
                        }
                    }
                    None => {}
                }

                let c_lines = function_code.split(";").collect::<Vec<&str>>();

                for cl in c_lines {
                    current_function.code.push(format!("{cl};"));
                }
            }
        } else {
            // Estamos dentro de uma função, precisamos capturar o conteúdo
            let mut current_line_content = String::new();
            let mut remaining_after_close = String::new();
            let mut found_end_in_this_line = false;

            for c in line.chars() {
                if !found_end_in_this_line {
                    if c == '{' {
                        steps_to_end += 1;
                    } else if c == '}' {
                        steps_to_end -= 1;
                        if steps_to_end == 0 {
                            found_end_in_this_line = true;
                            in_function = false;
                        }
                    }
  
                    if c == '}' && steps_to_end == 0 {
                        continue; // Não adiciona a chave de fechamento ao código da função
                    }
                    current_line_content.push(c);
                } else {
                    // Se a função fechou e ainda tem texto na linha, vai para o código global
                    remaining_after_close.push(c);
                }
            }

            // Adiciona o que pertence à função
            current_function.code.push(current_line_content);

            // Se a função acabou, salva e limpa
            if !in_function {
                functions.push(current_function);
                current_function = Function {
                    name: String::new(),
                    parameters: Vec::new(),
                    code: Vec::new(),
                };

                // Se sobrou código após o "}", adiciona ao código global
                if !remaining_after_close.trim().is_empty() {
                    code_widouth_functions.push(remaining_after_close);
                }
            }
        }
    }

    (code_widouth_functions, functions)
}
