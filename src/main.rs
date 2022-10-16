use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The command to run: add, checkout, commit, ...
    command: String,
}

fn main() {
    let args = Args::parse();
    if args.command == "add" {
        cmd_add(args)
    } else if args.command == "init" {
        cmd_init(args)
    } // else if ...
}

fn cmd_add(args: Args) {
    
}

fn cmd_init(args: Args) {
    todo!()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
