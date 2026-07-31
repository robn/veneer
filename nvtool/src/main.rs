use anystring::AnyString;
use nvpair::{PairList, PairValue};
use std::error::Error;
use std::io;

fn main() -> Result<(), Box<dyn Error>> {
    dump_pairlist(&nvpair::parse(io::stdin())?, 0);
    Ok(())
}

macro_rules! indent {
    ($level:expr) => {
        format_args!("{: >1$}", "", $level * 2)
    };
}

fn dump_pairlist(list: &PairList, indent: usize) {
    for pair in list.pairs() {
        let k = AnyString::from_c_str(pair.key());
        let v = pair.value();
        if matches!(v, PairValue::Boolean) {
            println!("{}{}", indent!(indent), k);
        } else {
            let vs: String = match v {
                PairValue::UInt64(v) => v.to_string(),
                PairValue::String(s) => AnyString::from_c_str(s).to_string(),
                PairValue::List(_) => "{".to_string(),
                PairValue::ListArray(v) => if v.is_empty() { "[]" } else { "[" }.to_string(),
                _ => format!("{:?}", v),
            };
            println!("{}{}: {}", indent!(indent), k, vs);
        }
        match v {
            PairValue::List(l) => {
                dump_pairlist(l, indent + 1);
                println!("{}}}", indent!(indent));
            }
            PairValue::ListArray(v) if !v.is_empty() => {
                let len = v.len();
                println!("{}{{", indent!(indent + 1));
                for (i, l) in v.iter().enumerate() {
                    dump_pairlist(l, indent + 2);
                    if i != len - 1 {
                        println!("{}}}, {{", indent!(indent));
                    }
                }
                println!("{}}}", indent!(indent + 1));
                println!("{}]", indent!(indent));
            }
            _ => {}
        }
    }
}
