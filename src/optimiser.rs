use crate::ast::Command;

#[derive(Debug, Clone)]
pub enum CommandFolded {
    Add(isize),
    Move(isize),
    Loop(Vec<CommandFolded>),
    Print,
    Read,
    SetZero,
    MoveValue { pos_rel: isize, mul: isize },
}

pub fn run(ast: &[Command]) -> Vec<CommandFolded> {
    ast.iter()
        .map(transform_single)
        .map(|com| optimize(&com))
        .collect()
}

pub fn transform(ast: &[Command]) -> Vec<CommandFolded> {
    ast.iter().map(transform_single).collect()
}

fn transform_single(command: &Command) -> CommandFolded {
    match command {
        Command::Add(n) => CommandFolded::Add(*n as isize),
        Command::Sub(n) => CommandFolded::Add(-(*n as isize)),
        Command::Left(n) => CommandFolded::Move(-(*n as isize)),
        Command::Right(n) => CommandFolded::Move(*n as isize),
        Command::Loop(sub_program) => CommandFolded::Loop(transform(sub_program)),
        Command::Read => CommandFolded::Read,
        Command::Print => CommandFolded::Print,
    }
}

fn optimize(command: &CommandFolded) -> CommandFolded {
    use CommandFolded::*;
    match command {
        CommandFolded::Loop(sub_program) => match &sub_program[..] {
            [Add(i)] if *i < 0 => SetZero,
            [Move(n), Add(i), Move(m), Add(j)] if *n == -*m && *j == -1 => MoveValue {
                pos_rel: *n,
                mul: *i,
            },
            sub_program => Loop(sub_program.iter().map(optimize).collect()),
        },
        command => command.to_owned(),
    }
}
