mod ast;

fn main() {

    let mut file = std::fs::read_to_string("./test.bf").unwrap();
    file.retain(|c| ['+', '-', '<', '>', '[', ']', '.', ','].contains(&c));

    let a = ast::parse(&file).ok().unwrap();
    ast::parse(a.0).unwrap_err();

    println!("{:?}", a);
}
