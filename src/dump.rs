use std::{fs::File, io::Write};

use crate::compiler::Compiler;

pub struct Dump {
    filename: String,
    content: String,
    compiler: Compiler,
}

impl Dump {
    pub fn new(filename: String, content: String, compiler: Compiler) -> Self {
        Dump {
            filename,
            content,
            compiler,
        }
    }

    pub fn dump(&mut self) {
        self.compiler.compile();
        for instruction in self.compiler.instructions.clone() {
            self.content.push_str(&format!("{}", instruction));
        }

        let mut file = File::create(&self.filename).unwrap();
        file.write_all(self.content.as_bytes()).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{compiler::Compiler, lexer::Lexing, parser::Parser};

    #[test]
    fn test_dump() {
        let filename = "test.dump".to_string();
        let content = "".to_string();
        let lexer = Lexing::new("1 + 2;");
        let mut parser = Parser::new(lexer);
        let program = parser.parse();
        let compiler = Compiler::new(program);
        let mut dump = Dump::new(filename, content, compiler);
        dump.dump();
    }
}
