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
                    .ok_or_else(|| ErrorContext::msg("missing required value").within(arg))?;
                number = value
                    .to_str()
                    .ok_or_else(|| {
                        ErrorContext::msg("invalid number")
                            .unexpected(Value(value))
                            .within(arg)
                    })?
                    .parse()
                    .map_err(|e| ErrorContext::msg(e).unexpected(Value(value)).within(arg))?;
            }
            Long("shout") => {
                shout = true;
            }
            Value(val) if thing.is_none() => {
                thing = Some(
                    val.to_str()
                        .ok_or_else(|| ErrorContext::msg("invalid string").unexpected(arg))?,
                );
            }
            Short("h") | Long("help") => {
                println!("Usage: hello [-n|--number=NUM] [--shout] THING");
                std::process::exit(0);
            }
            _ => {
                return Err(ErrorContext::msg("unexpected argument")
                    .unexpected(arg)
                    .within(Value(bin_name))
                    .into());
            }
        }
    }

    Ok(Args {
        thing: thing
            .ok_or_else(|| ErrorContext::msg("missing argument THING").within(Value(bin_name)))?
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
