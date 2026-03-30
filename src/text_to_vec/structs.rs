use std::collections::HashMap;

pub enum Types {
    Primitive(Primitive),
    Object(Box<Object>),
    Array(Vec<Types>),
}

pub enum Primitive {
    Null,
    Bool(bool),
    String(String),
    Float(f32),
    Double(f64),
    MiniInt(i8),
    Int(i16),
    LongInt(i32),
    LongLongInt(i64),
}

pub struct Object {
    fields: HashMap<String, Types>,
    methods: HashMap<String, FunctionDefinition>,
}

pub struct FunctionDefinition {
    params: HashMap<String, Types>,
    return_type: Types,
    body: String,
    scope: bool,
}

pub struct Environment {
    scopes: Vec<HashMap<String, Types>>,
}

impl Environment {
    //Cria o global
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    //Criar um novo galho
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    //Remove o ultimo galho
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn new_variable(&mut self, name: String, value: Types) {
        // Insere no escopo atual
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name, value);
        }
    }

    pub fn update_variable(&mut self, name: &str, value: Types) -> Result<(), String> {
        // Percorremos os escopos do último (topo/local) para o primeiro (global)
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                //Atualizamos o valor no escopo onde ele existe
                scope.insert(name.to_string(), value);
                return Ok(());
            }
        }

        // Se terminarmos o loop e não encontrarmos, é um erro de tempo de execução
        Err(format!("Erro: Variável '{}' não foi definida.", name))
    }

    pub fn get_variable(&self, name: &str) -> Option<&Types> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val);
            }
        }
        None
    }
}
