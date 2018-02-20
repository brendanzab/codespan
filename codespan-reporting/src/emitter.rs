use codespan::CodeMap;

use Diagnostic;

pub fn emit(codemap: &CodeMap, diagnostic: &Diagnostic) {
    println!("{}: {}", diagnostic.severity, diagnostic.message);
    for label in &diagnostic.labels {
        match codemap.find_file(label.span.start()) {
            None => if let Some(ref message) = label.message {
                println!("- {}", message)
            },
            Some(file) => {
                let (line, col) = file.location(label.span.start()).expect("location");

                print!("- {}:{}:{}", file.name(), line.number(), col.number());
                match label.message {
                    None => println!(),
                    Some(ref label) => println!(": {}", label),
                }
            },
        }
    }
}
