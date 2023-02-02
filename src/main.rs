fn main() {
    let args = match bg_next::get_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };
println!("HERE1main");

    if args.save.is_none() {

        println!("HERE2main");
        match bg_next::download_file(args).and_then(bg_next::run) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    } else {

        match bg_next::save(args.save.unwrap()) {
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
