//use crate::kill;
pub fn prepare_to_parse(content: String, is_debug: bool, strings: &mut Vec<String>) -> Vec<String> {
    let mut dentro_string: bool = false;
    let mut fechamento_aguardado: char = '"';
    let mut barra_inversa: u8 = 0;
    let special_chars: Vec<char> = vec!['n', 'r'];

    let pre_result: Vec<String> = content
        .lines()
        .map(|linha| {
            processar_linha(
                linha,
                &mut dentro_string,
                &mut fechamento_aguardado,
                &mut barra_inversa,
                &special_chars,
            )
        })
        .filter(|linha| !linha.trim().is_empty())
        .collect();

    if is_debug {
        println!("\n--- PRE-PARSING (Limpeza de Comentários) ---");
        for (idx, line) in pre_result.iter().enumerate() {
            println!("{:3} | {}", idx + 1, line);
        }
    }

    let mut saida: Vec<String> = Vec::new();

    let mut linha: Vec<char> = Vec::new();

    let fechamento = vec![';', '{', '}'];

    let mut string_atual: Vec<char> = Vec::new();

    for line in pre_result {
        for c in line.chars() {
            if dentro_string {
                //println!("Dentro de string: {}", c);

                if c == '\\' {
                    barra_inversa += 1;

                    if barra_inversa == 1 {
                        continue;
                    }

                    barra_inversa = 0;
                    string_atual.push(c);
                    string_atual.push(c);
                    continue;
                }

                string_atual.push(c);

                if barra_inversa == 1 {
                    if c == fechamento_aguardado {
                        barra_inversa = 0;

                        string_atual.pop();
                        string_atual.push('\\');
                        string_atual.push(c);

                        continue;
                    } else if special_chars.contains(&c) {
                        barra_inversa = 0;

                        string_atual.pop();
                        string_atual.push('\\');
                        string_atual.push(c);
                    }
                }

                if c == fechamento_aguardado {
                    let frase_final: String = string_atual.into_iter().collect();
                    dentro_string = false;
                    strings.push(frase_final);
                    string_atual = Vec::new();

                    let string_name = format!("*STRING:{}", strings.len() -1);
                    for l in string_name.chars() {
                        linha.push(l);
                    }
                }
                continue;
            }

            if c == '"' || c == '\'' || c == '`' {
                //println!("Iniciando string com: {}", c);

                dentro_string = true;
                fechamento_aguardado = c;
                string_atual.push(c);
                continue;
            }

            if linha.len() == 0 && c == ' ' {
                continue;
            }

            

            linha.push(c);

            if fechamento.contains(&c) {
                let sentenca: String = linha.into_iter().collect();
                if !sentenca.trim().is_empty() {
                    saida.push(sentenca);
                }

                linha = Vec::new();
                continue;
            };
        }
    }

    if is_debug {
        println!("\n--- POST-PARSING (Tokenização de Strings e Quebras) ---");
        for (idx, line) in saida.iter().enumerate() {
            println!("{:3} | {}", idx + 1, line);
        }
    }

    saida
}

fn processar_linha(
    linha: &str,
    dentro_string: &mut bool,
    fechamento_aguardado: &mut char,
    barra_inversa: &mut u8,
    special_chars: &Vec<char>,
) -> String {
    //println!("Processando linha: {}", linha);
    let mut resultado = String::new();
    let mut chars = linha.chars().peekable();

    let mut is_comment: u8 = 0; //Se o caractere "\" for encontrado, será adicionado +1. caso valor atinja 2, será considerado comentário. Qualquer coisa diferente de "\" zera o contador.

    let mut espacos: u8 = 0;

    if linha.len() > 0 {
        while let Some(c) = chars.next() {
            if c != '/' {
                is_comment = 0;
            }

            if c != ' ' {
                espacos = 0;
            }

            if *dentro_string {
                //println!("Dentro de string: {}", c);

                if c == '\\' {
                    *barra_inversa += 1;

                    if *barra_inversa == 1 {
                        continue;
                    }

                    *barra_inversa = 0;
                    resultado.push(c);
                    resultado.push(c);
                    continue;
                }

                resultado.push(c);

                if *barra_inversa == 1 {
                    if c == *fechamento_aguardado {
                        *barra_inversa = 0;

                        resultado.pop();
                        resultado.push('\\');
                        resultado.push(c);

                        continue;
                    } else if special_chars.contains(&c) {
                        *barra_inversa = 0;
                        resultado.pop();
                        resultado.push('\\');
                        resultado.push(c);
                    }
                }

                if c == *fechamento_aguardado {
                    *dentro_string = false;
                }
                continue;
            }

            if c == '"' || c == '\'' || c == '`' {
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

            if c == ' ' {
                espacos += 1;
                if espacos > 1 {
                    continue;
                }
            }

            if c == '\t' {
                continue;
            }

            resultado.push(c);
        }
    }
    resultado
}
