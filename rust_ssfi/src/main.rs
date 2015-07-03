extern crate ssfi_spmc;
extern crate getopts;

use std::env;

use ssfi_spmc::ssfi;

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
    opts.optopt("t", "", "number of threads", "NTHREADS");
    opts.optopt("d", "", "directory", "DIRECTORY");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    let nthreads: usize = matches.opt_str("t")
                                 .unwrap()
                                 .parse::<usize>()
                                 .unwrap();
    let directory = matches.opt_str("d").unwrap();

    // Run the program
    ssfi(nthreads, &directory);
}


