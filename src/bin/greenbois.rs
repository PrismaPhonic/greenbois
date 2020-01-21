use std::io::Write;
use std::process;
extern crate greenbois;
use structopt::StructOpt;

fn main() {
    let config = greenbois::Opts::from_args();

    if let Err(ref e) = greenbois::run(config) {
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);

        process::exit(1);
    };
}
