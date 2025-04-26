use std::{fs::File, io::BufReader};

use extendr_api::prelude::*;
use vcd::{Command, Header, Parser};

#[derive(Debug, IntoDataFrameRow)]
struct Vcd {
    pub date: Option<String>,
    pub version: Option<String>,
    pub time: Option<u32>,
    pub scale: Option<u32>,
}

/// Read a VCD file to a DataFrame
/// @export
#[extendr]
fn vcd_reader(file: String) -> Result<Dataframe<Vcd>> {
    eprintln!("{file}");

    let mut reader =
        Parser::new(BufReader::new(File::open(&file).map_err(|_| {
            Error::Other(format!("Could not open file {}", file))
        })?));

    let header: Header = reader
        .parse_header()
        .map_err(|_| Error::Other(format!("Could not parse VCD header for file {}", file)))?;

    while let Some(command) = reader.next().transpose().unwrap() {
        match command {
            Command::Timestamp(t) => println!("Time is {t}"),
            Command::ChangeScalar(i, v) => println!("{i} set to {v}"),
            Command::ChangeVector(i, v) => println!("{i} set to {v}"),
            Command::ChangeReal(i, v) => println!("{i} set to {v}"),
            Command::ChangeString(i, v) => println!("{i} set to {v}"),
            command => println!(
                "Unexpected {command:?} at line {line}",
                line = reader.line()
            ),
        }
    }

    let ret_test = vec![Vcd {
        date: None,
        version: None,
        time: None,
        scale: None,
    }];
    Ok(ret_test.into_dataframe()?)
}

/// Return string `"Hello world!"` to R.
/// @export
#[extendr]
fn hello_world() -> &'static str {
    "Hello world!"
}

// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod vcdreader;
    fn hello_world;
    fn vcd_reader;
}

#[cfg(test)]
pub mod test {
    use crate::vcd_reader;

    #[test]
    fn test_vcdreader() {
        let vcd = vcd_reader(format!(
            "{}/tests/testbench.vcd",
            env!("CARGO_MANIFEST_DIR")
        ));
    }
}
