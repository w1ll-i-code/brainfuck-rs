mod ast;

fn main() {
    let config: Config = match structopt::StructOpt::from_args_safe() {
        Ok(val) => val,
        Err(err) => {
            eprintln!("Could not parse commandline arguments:  {}", err.message);
            return;
        }
    };

    let a = ast::parse(&config).ok().unwrap();

    let a = ast::parse(&file).ok().unwrap();
    ast::parse(a.0).unwrap_err();

    println!("{:?}", a);
}
