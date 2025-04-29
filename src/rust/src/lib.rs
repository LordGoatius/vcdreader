#![feature(let_chains)]
use std::{fs::File, io::BufReader};

use extendr_api::prelude::*;
use vcd::{Command, Header, Parser, ScopeItem, SimulationCommand, TimescaleUnit, Var};

#[derive(Debug, IntoDataFrameRow)]
struct Vcd {
    pub date: Option<String>,
    pub version: Option<String>,
    pub time: Option<u32>,
    pub scale: Option<u32>,
}

fn add_all_names(names: &mut Vec<Var>, mut items: Vec<ScopeItem>) {
    while let Some(item) = items.pop() {
        match item {
            ScopeItem::Var(var) => {
                names.push(var);
            }
            ScopeItem::Scope(scope) => {
                add_all_names(names, scope.items);
            }
            // Ignore comments
            _ => (),
        }
    }
}

/// Read a VCD file to a DataFrame
/// @export
#[extendr]
fn vcd_reader(file: String) -> Result<Robj> {
    eprintln!("{file}");

    let mut reader =
        Parser::new(BufReader::new(File::open(&file).map_err(|_| {
            Error::Other(format!("Could not open file {}", file))
        })?));

    let header: Header = reader
        .parse_header()
        .map_err(|_| Error::Other(format!("Could not parse VCD header for file {}", file)))?;

    let items = header.items;
    let mut names = vec![];

    add_all_names(&mut names, items);

    // What we need to do
    // - continue until we reach a timestamp
    // - read values in until we reach a non change command
    // - if it's a timestamp, repeat
    // - if it's not, such as dumpall, ignore it?
    while let Some(command) = reader.next().transpose().unwrap() {
        match command {
            Command::Timestamp(t) => println!("Time is {t}"),
            Command::ChangeScalar(i, v) => println!("{i} set to {v}"),
            Command::ChangeVector(i, v) => println!("{i} set to {v}"),
            Command::ChangeReal(i, v) => println!("{i} set to {v}"),
            Command::ChangeString(i, v) => println!("{i} set to {v}"),
            Command::Comment(_) => continue,
            Command::Begin(SimulationCommand::Dumpall) => continue,
            command => println!(
                "Unexpected {command:?} at line {line}",
                line = reader.line()
            ),
        }
        break;
    }

    let date_r = r!(header.date);
    let version_r = r!(header.version);
    let timescale_r = header.timescale;
    let time_r = r!(timescale_r.map(|x| x.0));

    let scale_r = r!(timescale_r.map(|x| match x.1 {
        TimescaleUnit::S => 1,
        TimescaleUnit::MS => 3,
        TimescaleUnit::US => 6,
        TimescaleUnit::NS => 9,
        TimescaleUnit::PS => 12,
        TimescaleUnit::FS => 15,
    }));

    let name: Vec<_> = names.iter().map(|var| var.reference.clone()).collect();
    let codes: Vec<_> = names.iter().map(|var| format!("{}", var.code)).collect();
    //let ids: Vec<_> = names.iter().map(|var| var.index.map(|ind_ref| ));

    let names_r = r!(name);
    let codes_r = r!(codes);

    Ok(list!(
        filename = file,
        date = date_r,
        version = version_r,
        time = time_r,
        scale = scale_r,
        vars = names_r,
        codes = codes_r,
    )
    .into())
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
