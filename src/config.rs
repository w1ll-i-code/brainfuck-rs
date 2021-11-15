use std::path::PathBuf;
use std::str::FromStr;
use structopt::clap::{Error, ErrorKind};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Config {
    #[structopt(parse(from_os_str))]
    pub input_file: PathBuf,
    #[structopt(short, long = "output", default_value = "a.out")]
    pub output_file: String,
    #[structopt(short = "O", long = "optimization", default_value = "Normal")]
    pub optimisation_level: OptimisationLevel,
    #[structopt(short, long, default_value = "I64")]
    pub cell_size: CellSize,
}

#[derive(Debug, PartialEq)]
pub enum OptimisationLevel {
    Off,
    Normal,
    Max,
}

#[derive(Debug)]
pub enum CellSize {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
}

impl FromStr for OptimisationLevel {
    type Err = structopt::clap::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Off" => Ok(OptimisationLevel::Off),
            "Normal" => Ok(OptimisationLevel::Normal),
            "Max" => Ok(OptimisationLevel::Max),
            _ => Err(Error::with_description(
                &format!(
                    "No optimisation level <{}>. Available levels: [Off, Normal, Max].",
                    s
                ),
                ErrorKind::InvalidValue,
            )),
        }
    }
}

impl FromStr for CellSize {
    type Err = structopt::clap::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "I8" => Ok(CellSize::I8),
            "I16" => Ok(CellSize::I16),
            "I32" => Ok(CellSize::I32),
            "I64" => Ok(CellSize::I64),
            "U8" => Ok(CellSize::U8),
            "U16" => Ok(CellSize::U16),
            "U32" => Ok(CellSize::U32),
            "U64" => Ok(CellSize::U64),
            _ => Err(Error::with_description(
                &format!(
                "No cell_size <{}> available. Available sizes: [I8, I16, I32, I64, U8, U16, U32, U64]",
                s
            ),
             ErrorKind::InvalidValue
            )),
        }
    }
}
