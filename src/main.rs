fn main() {
    let args = match synth::get_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    if args.save.is_none() && args.load.is_none() {

        match synth::download_file(args).and_then(synth::run) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    } else if args.load.is_none() {

        match synth::save(args.save.unwrap()) {
            Ok(_res) => {

            },
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        };
    } else {
        match synth::load(args.load.unwrap()) {
            Ok(_res) => {
            },
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);

            }
        }
    }
}
