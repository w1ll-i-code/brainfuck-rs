use crate::config::{Config, OptimisationLevel};
use crate::llvm_generation::Generator;
use crate::optimiser::transform;
use inkwell::context::Context;

mod ast;
mod config;
mod llvm_generation;
mod optimiser;

fn main() {
    let config: Config = match structopt::StructOpt::from_args_safe() {
        Ok(val) => val,
        Err(err) => {
            eprintln!("Could not parse commandline arguments:  {}", err.message);
            return;
        }
    };

    let a = ast::parse(&config).ok().unwrap();

    let a = match config.optimisation_level {
        OptimisationLevel::Max => optimiser::run(&a),
        _ => optimiser::transform(&a),
    };

    let context = Context::create();
    let generator = Generator::new(&context, &config);
    generator.generate(&a);
    generator.write_to_file(&config.output_file);

    println!("{:#?}", config);
    println!("{:?}", a);
}
