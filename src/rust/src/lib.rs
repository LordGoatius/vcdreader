use std::{fs::File, io::BufReader};

use extendr_api::prelude::*;
use vcd::Parser;

#[derive(Debug, IntoDataFrameRow)]
struct Empty {
    x: i32
}

/// Read a VCD file to a DataFrame
/// @export
#[extendr]
fn vcd_reader(file: String) -> Result<Dataframe<Empty>> {
    let mut reader =
        Parser::new(BufReader::new(File::open(&file).map_err(|_| {
            Error::Other(format!("Could not open file {}", file))
        })?));

    let header = reader
        .parse_header()
        .map_err(|_| Error::Other(format!("Could not parse VCD header for file {}", file)))?;

    let ret_test = vec![Empty { x: 31 }];
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
