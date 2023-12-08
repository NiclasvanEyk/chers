use std::io::Write;

pub fn prompt(question: &str) -> String {
    let mut x = String::new();
    print!("{}", question);
    std::io::stdout().flush().expect("Could not flush stdout");

    std::io::stdin()
        .read_line(&mut x)
        .expect("Could not read input");

    x
}
