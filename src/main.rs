fn main() {
    let args = match synth::get_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };
println!("HERE1main");

    if args.save.is_none() {

        println!("HERE2main");
        match synth::download_file(args).and_then(synth::run) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    } else {

        match synth::save(args.save.unwrap()) {
            Ok(_res) => {
            
                println!("HERE3main");
            },
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        };
    }
}
