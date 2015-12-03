extern crate ssfi_rust;
extern crate getopts;

use std::env;

use ssfi_rust::ssfi;

use getopts::Options;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    let mut b_print = false;
    opts.optopt("t", "", "number of threads", "NTHREADS");
    opts.optopt("d", "", "directory", "DIRECTORY");
    opts.optopt("p", "", "print", "PRINT");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    if matches.opt_present("p") {
        b_print = true;
    }
    let nthreads: usize = matches.opt_str("t")
                                 .unwrap()
                                 .parse::<usize>()
                                 .unwrap();
    let directory = matches.opt_str("d").unwrap();

    // Run the program
    ssfi(nthreads, &directory, b_print);
}


