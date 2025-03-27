use lexarg::ErrorContext;
use lexarg::Result;

struct Args {
    thing: String,
    number: u32,
    shout: bool,
}

fn parse_args() -> Result<Args> {
    use lexarg::prelude::*;

    let mut thing = None;
    let mut number = 1;
    let mut shout = false;
    let raw = std::env::args_os().collect::<Vec<_>>();
    let mut parser = lexarg::Parser::new(&raw);
    let bin_name = parser
        .next_raw()
        .expect("nothing parsed yet so no attached lingering")
        .expect("always at least one");
    while let Some(arg) = parser.next_arg() {
        match arg {
            Short("n") | Long("number") => {
                let value = parser
                    .next_flag_value()
                    .ok_or_missing(Value(std::ffi::OsStr::new("NUM")))
                    .within(arg)?;
                number = value.parse().within(arg)?;
            }
            Long("shout") => {
                shout = true;
            }
            Value(val) if thing.is_none() => {
                thing = Some(val.string("string")?);
            }
            Short("h") | Long("help") => {
                println!("Usage: hello [-n|--number=NUM] [--shout] THING");
                std::process::exit(0);
            }
            _ => {
                return Err(ErrorContext::msg("unexpected argument")
                    .unexpected(arg)
                    .into());
            }
        }
    }

    Ok(Args {
        thing: thing
            .ok_or_missing(Value(std::ffi::OsStr::new("THING")))
            .within(Value(bin_name))?
            .to_owned(),
        number,
        shout,
    })
}

fn main() -> Result<()> {
    let args = parse_args()?;
    let mut message = format!("Hello {}", args.thing);
    if args.shout {
        message = message.to_uppercase();
    }
    for _ in 0..args.number {
        println!("{message}");
    }
    Ok(())
}
