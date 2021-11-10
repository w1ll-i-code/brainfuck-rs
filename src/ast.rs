use nom::{IResult, Parser, InputLength};
use nom::bytes::complete::tag;
use nom::multi::many1;
use nom::error::ParseError;

#[derive(Debug)]
pub enum Command {
    Add(usize),
    Sub(usize),
    Left(usize),
    Right(usize),
    Loop(Vec<Command>),
    Read,
    Print,
}

pub fn parse(input: &str) -> IResult<&str, Vec<Command>> {
    many1(parse_command)(input)
}

fn parse_command(input: &str) -> IResult<&str, Command> {
    accumulate(parse_add, Command::Add)(input)
        .or_else(|_| accumulate(parse_sub, Command::Sub)(input))
        .or_else(|_| accumulate(parse_left, Command::Left)(input))
        .or_else(|_| accumulate(parse_right, Command::Right)(input))
        .or_else(|_| parse_read(input))
        .or_else(|_| parse_print(input))
        .or_else(|_| parse_loop(input))
}

pub fn accumulate<I, O, E, F, F2>(f: F, f2: F2) -> impl FnOnce(I) -> IResult<I, O, E>
    where
        I: Clone + InputLength,
        F: Parser<I, O, E>,
        F2: FnOnce(usize) -> O ,
        E: ParseError<I>,
{
    move |input| many1(f)(input).map(|(i, b)| (i, f2(b.len()) ))
}

fn parse_loop(input: &str) -> IResult<&str, Command> {
    let (input, _) = tag("[")(input)?;
    let (input, commands) = many1(parse_command)(input)?;
    let (input, _) = tag("]")(input)?;
    IResult::Ok((input, Command::Loop(commands)))
}

macro_rules! parser {
    ($name:ident, $tag:expr, $result:expr) => {
        fn $name(input: &str) -> IResult<&str, Command> {
            let (input, _) = tag($tag)(input)?;
            IResult::Ok((input, $result))
        }
    }
}

parser!(parse_add, "+", Command::Add(1));
parser!(parse_sub, "-", Command::Sub(1));
parser!(parse_left, "<", Command::Left(1));
parser!(parse_right, ">", Command::Right(1));
parser!(parse_read, ",", Command::Read);
parser!(parse_print, ".", Command::Print);
