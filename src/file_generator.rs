pub use crate::prelude::*;

pub fn generate_file(fn_name: impl Into<String>, args: Vec<String>, data: &Vec<Part>) -> Vec<String> {
    let fn_name = fn_name.into();

    let args = args.join(", ");
    let mut code_lines: Vec<String> = Vec::new();
    code_lines.push(format!("fn {fn_name}({args}) -> Result<String, Box<dyn std::error::Error>> {{"));
    code_lines.push("use std::fmt::Write;".to_string());
    code_lines.push("let mut output_buffer = String::new();".to_string());

    for part in data {
        match part {
            Part::Code(code) => {
                code_lines.push(code.to_string());
            }
            Part::EchoCode(code) => {
                code_lines.push(format!("\twrite!(output_buffer, \"{{}}\", {{ {code} }})?;"));
            }
            Part::Text(text) => {
                code_lines.push(format!("write!(output_buffer, \"{{}}\", \"{}\")?;", text.escape_default()));
            }
        }
    }

    code_lines.push("Ok(output_buffer)".to_string());

    code_lines.push("}".to_string());

    code_lines
}

pub fn format_code(code: &str) -> String {
    let syntax_tree = syn::parse_file(code).unwrap();
    let formatted = prettyplease::unparse(&syntax_tree);
    formatted
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;
    use crate::file_generator::{format_code, generate_file};
    use crate::prelude::*;

    #[test]
    fn it_works() {
        let file = read_to_string("src/test-files/file_generator_01.plt").unwrap();

        let mut fsa = TextCodeFSA::new();

        let result = fsa.run(file);

        let generated_file = generate_file("test_template", result);

        let code = generated_file.join("\r\n");

        println!("{}", format_code(&code));
    }
}