// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023-2025, Rob Norris <robn@despairlabs.com>

use bytesize::ByteSize;
use std::error::Error;
use tabled::{builder::Builder, settings::Style};

enum Field<'a> {
    Bytes(&'a str),
    String(&'a str),
}

impl Field<'_> {
    fn name(&self) -> &str {
        match self {
            Field::Bytes(s) => s,
            Field::String(s) => s,
        }
    }
}

const FIELDS: &[Field] = &[
    Field::Bytes("used"),
    Field::Bytes("available"),
    Field::Bytes("referenced"),
    Field::String("mountpoint"),
];

fn main() -> Result<(), Box<dyn Error>> {
    let z = veneer::open()?;

    let mut tb = Builder::default();
    tb.push_record(
        ["name".to_string()]
            .into_iter()
            .chain(FIELDS.iter().map(|f| f.name().to_string()))
            .map(|ref s| s.to_string())
            .collect::<Vec<_>>(),
    );

    for pool in z.pools()? {
        for dataset in pool.datasets()? {
            tb.push_record(
                [dataset.name().to_string()]
                    .into_iter()
                    .chain(
                        FIELDS
                            .iter()
                            .map(|s| match s {
                                Field::Bytes(s) => dataset
                                    .get_prop_u64(s)
                                    .map(|s| s.map(|n| ByteSize::b(n).display().iec().to_string())),
                                Field::String(s) => {
                                    dataset.get_prop_string(s).map(|s| s.map(|v| v.to_string()))
                                }
                            })
                            .map(|r| match r {
                                Ok(Some(s)) => s,
                                Ok(None) => "-".to_string(),
                                Err(_) => "?".to_string(),
                            }),
                    )
                    .collect::<Vec<_>>(),
            );
        }
    }

    let table = tb.build().with(Style::rounded()).to_string();
    println!("{}", table);

    Ok(())
}
