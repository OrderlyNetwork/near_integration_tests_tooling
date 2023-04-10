use crate::error::TestError;
use std::{
    fs::File,
    io::{self, Write},
    path::PathBuf,
};

/// Defines the output destination for printing the statistic
/// * Console - will print to the Console only
/// * File - will print to the specified file
/// * Mixed - will print both to the Console and specified file
#[derive(Debug)]
pub enum ModePrinter {
    Console,
    File(PathBuf),
    Mixed(PathBuf),
}

impl ModePrinter {
    pub fn print(&self, buf: &[u8]) -> Result<(), TestError> {
        match self {
            Self::Console => Self::select_stdout().write_all(buf),
            Self::File(path) => Self::select_file_output(path)?.write_all(buf),
            Self::Mixed(path) => Self::select_file_output(path)?
                .write_all(buf)
                .and_then(|_| Self::select_stdout().write_all(buf)),
        }
        .map_err(|err| TestError::Custom(err.to_string()))
    }

    fn select_file_output(path: &PathBuf) -> Result<Box<dyn Write>, TestError> {
        File::create(path)
            .map(|f| Box::new(f) as Box<dyn Write>)
            .map_err(|err| TestError::Custom(err.to_string()))
    }

    fn select_stdout() -> Box<dyn Write> {
        Box::new(io::stdout())
    }
}

// It was decided to print both to stdout and file by default instead of to file only
impl From<PathBuf> for ModePrinter {
    fn from(value: PathBuf) -> Self {
        ModePrinter::Mixed(value)
    }
}

// The default is console only print
impl Default for ModePrinter {
    fn default() -> Self {
        ModePrinter::Console
    }
}
