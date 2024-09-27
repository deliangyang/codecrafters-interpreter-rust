use std::{collections::HashMap, fs};

use crate::{
    ast::{self, Progam, Stmt},
    lexer::Lexing,
    parser::Parser,
};

pub struct Imports {
    imports: HashMap<String, ast::Progam>,
    program: ast::Progam,
}

impl Imports {
    pub fn new(program: ast::Progam) -> Self {
        Imports {
            imports: HashMap::new(),
            program,
        }
    }

    pub fn load(&mut self) -> Progam{
        let imports = self.load_all(self.program.clone());
        if let Some(imports) = imports {
            for (k, v) in imports {
                self.insert(k, v);
            }
        }
        let mut progs = vec![];
        for (_, v) in self.imports.clone() {
            progs.extend(v);
        }
        progs.extend(self.program.clone());
        progs
    }

    fn load_all(&mut self, progam: ast::Progam) -> Option<HashMap<String, Progam>> {
        let imports = progam.iter().filter(|stmt| match stmt {
            Stmt::Import(_) => true,
            _ => false,
        });
        if imports.clone().count() == 0 {
            return None;
        }
        let mut progs = HashMap::new();
        for import in imports.clone() {
            match import {
                Stmt::Import(s) => {
                    let current_dir = std::env::current_dir().unwrap();
                    let filename = format!("{}.lox", s);
                    let file_contents =
                        fs::read_to_string(current_dir.join(filename.clone())).unwrap();
                    if file_contents.is_empty() {
                        panic!("Error reading file: {}", filename.clone());
                    }
                    if self.imports.contains_key(&filename) {
                        continue;
                    }
                    let lex = Lexing::new(&file_contents);
                    let mut parse = Parser::new(lex);
                    let program = parse.parse();
                    if parse.has_errors() {
                        panic!("Error parsing file: {}", filename);
                    }
                    let imports = self.load_all(program.clone());
                    if let Some(imports) = imports {
                        for (k, v) in imports {
                            progs.insert(k, v);
                        }
                    }
                    progs.insert(filename.clone(), program);
                }
                _ => {}
            }
        }
        Some(progs)
    }

    fn insert(&mut self, name: String, program: ast::Progam) {
        self.imports.insert(name, program);
    }

    pub fn get(&self, name: &str) -> Option<&ast::Progam> {
        self.imports.get(name)
    }
}
