use clap::{Command, Arg};
use regex::Regex;
use std::error::Error;
use std::path::Path;
use std::io::prelude::*;
use rand::Rng;
use std::time::Instant;
mod errors;
mod websites;
use crate::websites::{Website, Wallbase};



//TODO: Create a config folder on install and provide a sample config file
//allow a user to give a path to a config file via arg
//use a default config if api key is provided via args
//pull the src_dir from the config file so it can be changed
//handle errors appropriately throughout
//place nitrogen/flavours into functions so they can be swapped with pywal and feh

pub struct InputArgs {
    color: Option<String>,
    query: Option<String>,
    resolution: Option<String>,
    pub save: Option<String>,
    pub load: Option<String>,
}

static SRC_DIR: &str = "/home/huginn/.synth";

pub fn get_args() -> Result<InputArgs, Box<dyn Error>> {

    let matches = Command::new("synth")
        .version("0.1.0")
        .author("Zach Shore")
        .about("Wallpaper and theme generator for flavours")
        .arg(
            Arg::new("color")
            .short('c')
            .long("color")
            .help("Searches for backgrounds similar to the selected color. Use help to see a list of options.")
            .num_args(1),
        )
        .arg(
            Arg::new("query")
            .short('q')
            .long("query")
            .help("Searches terms for appropriate backgrounds")
            .num_args(1)
            .conflicts_with_all(["save","load"])
        )
        .arg(
            Arg::new("save")
            .short('s')
            .long("save")
            .help("Save the current wallpaper and theme under the specified name")
            .num_args(1)
            .conflicts_with_all(["color","query","load"])
        )
        .arg(
            Arg::new("load")
            .short('l')
            .long("load")
            .help("Load a previously saved theme and wallpaper")
            .num_args(1)
            .conflicts_with_all(["color","query","save"])
        )
        .get_matches();


    let color = matches.get_one::<String>("color").cloned();
    let query = matches.get_one::<String>("query").cloned();
    let save = matches.get_one::<String>("save").cloned();
    let load = matches.get_one::<String>("load").cloned();

    let resolution = get_resolution();

    Ok(InputArgs {
        color,
        query,
        resolution,
        save,
        load,
    })
}

fn get_resolution() -> Option<String> {

    let display_info_utf8 = std::process::Command::new("xdpyinfo").output().unwrap();
    let display_info_str = String::from_utf8(display_info_utf8.stdout).unwrap();
    let display_info = display_info_str.as_str();
    let reg_res = Regex::new(r"dimensions:\s+([\S]+)").unwrap();
    let capture = reg_res.captures(display_info);
    match capture {
        Some(c) => c.get(1).map(|m| m.as_str().to_string()),
        None => { 
            println!("Could not determine display resolution!");
            None
        }
    }
}

pub fn download_file(inputargs: InputArgs) -> Result<Vec<String>, Box<dyn Error>> {

    let wallbase = Wallbase::build("config.toml".to_owned(), inputargs)?;
    let image_uri = wallbase.get_image();

    let gen = "/generated";
    let mydir = format!("{}{}", SRC_DIR, gen);
    let mut image_path = format!("{}{}{}", SRC_DIR,gen, gen);
    let files_path = format!("{}{}{}{}", SRC_DIR, gen, gen, ".yml");


    remove_all_files(&Path::new(&mydir));

    let filetypes: Vec<&str> = vec![".jpg", ".png"];
    for filetype in filetypes {
        if image_uri.contains(filetype) {
            image_path.push_str(filetype);
            break;
        }
    }

    std::fs::write(&image_path, reqwest::blocking::get(image_uri)?.bytes()?)?;
   
    Ok(vec![files_path, image_path])//This is weird. How else can I do this?
}

fn remove_all_files(dir: &Path) {
    if dir.is_dir() {
        std::fs::read_dir(dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().unwrap().is_file())
            .for_each(|entry| {
                std::fs::remove_file(entry.path()).unwrap();
            });
    }
}
                    

pub fn run(files_path: Vec<String>) -> Result<(), Box<dyn Error>>{

    let cmd1 = std::process::Command::new("nitrogen")
        .args(["--set-auto", files_path[1].as_str()]).output()?;
    
    let mut file = std::fs::File::create(&files_path[0])?;
    let cmd2 = std::process::Command::new("flavours")
        .args(["generate","dark","--stdout",files_path[1].as_str()]).output()?;
    
    file.write_all(&cmd2.stdout)?;

    let cmd3 = std::process::Command::new("flavours")
        .stdin(std::fs::File::open(&files_path[0])?)
        .args(["apply", "--stdin"]).output()?;

    for cmd in vec![cmd1,cmd2,cmd3] {
        let temp = String::from_utf8(cmd.stdout).unwrap();
        if temp != "" {
            println!("{}", temp);
        }
    }

    Ok(())
}

fn create_theme_id() -> String {

    let now = Instant::now();
    let timestamp = now.elapsed().as_secs();
    let random = rand::thread_rng().gen::<i32>();
    format!("{}_{}", timestamp, random)

}


//pywal works on base16 as well. Implement alternate commands for FEH/Nitrogen Pywal/flavours
pub fn save(theme_name: String) -> Result<(), Box<dyn Error>> {

    //let app_folder_base = "/home/huginn/.synth".to_string(); //Needs to be a default value or come from config
    let mut app_folder = SRC_DIR.to_string();

    if let Ok(metadata) = std::fs::metadata(&app_folder) {
        if !metadata.is_dir() {
            println!("File with the same name exists!");
        }
    } else { 
        std::fs::create_dir(&app_folder)?;

    }

    app_folder.push_str("/");
    app_folder.push_str(&theme_name);

    if let Ok(metadata) = std::fs::metadata(&app_folder) {
        if !metadata.is_dir() {
            println!("File with the same name exists!");
        }
    } else { 
        std::fs::create_dir(&app_folder)?;
    }

    //let theme_id = create_theme_id();
    copy_folder_and_rename_files(SRC_DIR, &theme_name)?;

    //Create a default folder path for all of this. Allow this to be changed to config. Write in
    //readme about using .local/share/flavours/base16 as default path to add themes there
    //make sure to still use generated/generated.yaml for randomized themes before saving
                                                                                        
    Ok(())
}

pub fn load(theme_name: String) -> Result<(), Box<dyn Error>> {
    //Load yaml into flavours apply --stdin from the path
    //if path is under ~/.local/share/flavours/base16/schemes/{theme_name}/{theme_name}.yaml it will
    //show up under flavours list

    let extensions = vec!["jpg","png","yml"];
    for ext in extensions {
        let file_path = format!("{}/{}/{}.{}",SRC_DIR, theme_name, theme_name, ext);
        if let Ok(res) = std::fs::metadata(&file_path) {
            match res.is_file() {
                true => {
                    if ext == "yml" {
                        let file = std::fs::File::open(&file_path)?;
                        let _cmd = std::process::Command::new("flavours")
                            .stdin(std::process::Stdio::from(file))
                            .args(["apply", "--stdin"])
                            .spawn()?; 
                    } else {
                        let _cmd = std::process::Command::new("nitrogen")
                            .args(["--set-auto", &file_path])
                            .spawn()?;
                    }
                }

                false => {
                }
            }

        }
    }

    Ok(())

}


fn copy_folder_and_rename_files(src_dir: &str, theme_name: &str) -> std::io::Result<()> {
    
    let src_str = format!("{}/{}", SRC_DIR, "generated");
    let src_path = Path::new(&src_str);
    let dest_str = format!("{}/{}",src_dir, theme_name);
    let dest_path = Path::new(&dest_str);

    // Create the destination folder if it doesn't exist
    if !dest_path.exists() {
        std::fs::create_dir_all(dest_path)?;
    }

    // Loop through the items in the source folder
    for entry in std::fs::read_dir(src_path)? {
        let entry = entry?;
        let src = entry.path();
        let path = entry.path();
        let extension = path.extension().unwrap().to_str().unwrap();
        let new_path = format!("{}/{}{}", dest_path.to_string_lossy(), theme_name, extension );
        std::fs::copy(entry.path(), new_path)?;
    }

    Ok(())
}

