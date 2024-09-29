use std::{collections::HashMap, fs, path::PathBuf};

use crate::{
    ast::{self, Program, Stmt},
    lexer::Lexing,
    parser::Parser,
};

pub struct Imports {
    imports: HashMap<String, ast::Program>,
    program: ast::Program,
    current_dir: std::path::PathBuf,
}

impl Imports {
    pub fn new(program: ast::Program, current_dir: PathBuf) -> Self {
        Imports {
            imports: HashMap::new(),
            program,
            current_dir,
        }
    }

    pub fn load(&mut self) -> Program{
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

    fn load_all(&mut self, program: ast::Program) -> Option<HashMap<String, Program>> {
        let imports = program.iter().filter(|stmt| match stmt {
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
                    let filename = format!("{}.lox", s);
                    let file_contents =
                        fs::read_to_string(self.current_dir.join(filename.clone())).unwrap();
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

    fn insert(&mut self, name: String, program: ast::Program) {
        self.imports.insert(name, program);
    }

    pub fn get(&self, name: &str) -> Option<&ast::Program> {
        self.imports.get(name)
    }
}
