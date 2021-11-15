use crate::config::{Config, OptimisationLevel};
use nom::bytes::complete::tag;
use nom::error::ParseError;
use nom::multi::many1;
use nom::sequence::delimited;
use nom::{IResult, InputLength, InputTakeAtPosition, Parser};

macro_rules! parse_command {
    ($fn: expr) => {
        delimited(multicomment0, $fn, multicomment0)
    };
}

// possible future:
//     # for comments
//     ( and ) for imports
//     ! for error state (with message?)
const KEYWORDS: [char; 8] = ['+', '-', '<', '>', '[', ']', '.', ','];

type ParserType = fn(&str) -> IResult<&str, Command>;

#[derive(Debug, Clone)]
pub enum Command {
    Add(usize),
    Sub(usize),
    Left(usize),
    Right(usize),
    Loop(Vec<Command>),
    Read,
    Print,
}

pub fn parse(config: &Config) -> Result<Vec<Command>, String> {
    let file = std::fs::read_to_string("./test.bf").unwrap();

    let parser = match config.optimisation_level {
        OptimisationLevel::Off => parse_command,
        _ => parse_command_optimized,
    };

    let result = match many1(parser)(&file as &str) {
        Ok(("", program)) => Ok(program),
        Ok(_) => Err("Unknown error while parsing: Could not parse the whole file".to_owned()),
        Err(e) => Err(e.to_string()),
    };

    result
}

fn multicomment0(input: &str) -> IResult<&str, &str> {
    input.split_at_position_complete(is_command)
}

fn is_command(c: char) -> bool {
    KEYWORDS.contains(&c)
}

fn parse_command(input: &str) -> IResult<&str, Command> {
    parse_command!(parse_add)(input)
        .or_else(|_| parse_command!(parse_sub)(input))
        .or_else(|_| parse_command!(parse_left)(input))
        .or_else(|_| parse_command!(parse_right)(input))
        .or_else(|_| parse_command!(parse_read)(input))
        .or_else(|_| parse_command!(parse_print)(input))
        .or_else(|_| parse_command!(parse_loop(parse_command))(input))
}

fn parse_command_optimized(input: &str) -> IResult<&str, Command> {
    accumulate(parse_command!(parse_add), Command::Add)(input)
        .or_else(|_| accumulate(parse_command!(parse_sub), Command::Sub)(input))
        .or_else(|_| accumulate(parse_command!(parse_left), Command::Left)(input))
        .or_else(|_| accumulate(parse_command!(parse_right), Command::Right)(input))
        .or_else(|_| parse_command!(parse_read)(input))
        .or_else(|_| parse_command!(parse_print)(input))
        .or_else(|_| parse_command!(parse_loop(parse_command_optimized))(input))
}

pub fn accumulate<I, O, E, F, F2>(f: F, f2: F2) -> impl FnOnce(I) -> IResult<I, O, E>
where
    I: Clone + InputLength,
    F: Parser<I, O, E>,
    F2: FnOnce(usize) -> O,
    E: ParseError<I>,
{
    move |input| many1(f)(input).map(|(i, b)| (i, f2(b.len())))
}

fn parse_loop(f: ParserType) -> impl FnMut(&str) -> IResult<&str, Command> {
    move |input| {
        let (input, sub_program) = delimited(tag("["), many1(f), tag("]"))(input)?;
        Ok((input, Command::Loop(sub_program)))
    }
}

macro_rules! parse_char_to {
    ($name:ident, $tag:expr, $result:expr) => {
        fn $name(input: &str) -> IResult<&str, Command> {
            let (input, _) = tag($tag)(input)?;
            IResult::Ok((input, $result))
        }
    };
}

parse_char_to!(parse_add, "+", Command::Add(1));
parse_char_to!(parse_sub, "-", Command::Sub(1));
parse_char_to!(parse_left, "<", Command::Left(1));
parse_char_to!(parse_right, ">", Command::Right(1));
parse_char_to!(parse_read, ",", Command::Read);
parse_char_to!(parse_print, ".", Command::Print);
