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


pub struct InputArgs {
    color: Option<String>,
    query: Option<String>,
    resolution: Option<String>,
    pub save: Option<String>,
    pub load: Option<String>,
}

pub fn get_args() -> Result<InputArgs, Box<dyn Error>> {

    let matches = Command::new("bg_next")
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

    let files_path = "/home/huginn/.synth/generated/generated".to_string();
    let mut image_path = files_path.to_owned();


//    std::fs::write(&image_path, reqwest::blocking::get(&image_uri)?.bytes()?)?;

    
    let filetypes: Vec<&str> = vec![".jpg", ".png"];
    for filetype in filetypes {
        if image_uri.contains(filetype) {
            image_path.push_str(filetype);
            break;
        }
    }

    std::fs::write(&image_path, reqwest::blocking::get(image_uri)?.bytes()?)?;
   
    Ok(vec![files_path, image_path])
}

pub fn run(files_path: Vec<String>) -> Result<(), Box<dyn Error>>{

    let cmd1 = std::process::Command::new("nitrogen")
        .args(["--set-auto", files_path[1].as_str()]).output()?;
    let cmd2 = std::process::Command::new("flavours")
        .args(["generate","dark",files_path[1].as_str()]).output()?;
    let mut file = std::fs::File::create(&files_path[0])?;
    
    file.write_all(&cmd2.stdout)?;
    let cmd3 = std::process::Command::new("flavours")
        .args(["apply", "generated"]).output()?;

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

    let app_folder_base = "/home/huginn/.synth".to_string(); //Needs to be a default value or come from config
    let mut app_folder = app_folder_base.to_owned();

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
    copy_folder_and_rename_files("/home/huginn/.synth", &theme_name)?;

    //Create a default folder path for all of this. Allow this to be changed to config. Write in
    //readme about using .local/share/flavours/base16 as default path to add themes there
    //make sure to still use generated/generated.yaml for randomized themes before saving
                                                                                        
    Ok(())
}

pub fn load(theme_name: String) -> Result<(), Box<dyn Error>> {
//Load yaml into flavours apply --stdin from the path
//if path is under ~/.local/share/flavours/base16/schemes/{theme_name}/{theme_name}.yaml it will
//show up under flavours list

    let src_dir = "/home/huginn/.synth";
    let extensions = vec!["jpg","png","yml"];
    for ext in extensions {
        let file_path = format!("{}/{}/{}.{}",src_dir, theme_name, theme_name, ext);
        println!("{}", file_path);
        if let Ok(res) = std::fs::metadata(&file_path) {
            println!("HERELOAD1");
            match res.is_file() {
                true => {

                    println!("HERELOAD2");
                    if ext == "yml" {
                        let file = std::fs::File::open(&file_path)?;
                        //let contents = std::fs::read_to_string(file_path);
                        let _cmd = std::process::Command::new("flavours")
                            .stdin(std::process::Stdio::from(file))
                            .args(["apply", "--stdin"])
                            .spawn()?; 
                    } else {

                        println!("HERELOAD3");
                        let _cmd = std::process::Command::new("nitrogen")
                            .args(["--set-auto", &file_path])
                            .spawn()?;
                    }
                }

                false => {
                }
            }

            println!("HERELOAD4");
        }
    }

    println!("HERELOAD5");
    Ok(())

}


fn copy_folder_and_rename_files(src_dir: &str, theme_name: &str) -> std::io::Result<()> {
    let src_path = Path::new("/home/huginn/.synth/generated");
    let dest_str = format!("{}/{}",src_dir, theme_name);
    let dest_path = Path::new(&dest_str);

        println!("HERE4");
    // Create the destination folder if it doesn't exist
    if !dest_path.exists() {
        std::fs::create_dir_all(dest_path)?;

        println!("HERE5");
    }

    println!("{}", theme_name);
    // Loop through the items in the source folder
    for entry in std::fs::read_dir(src_path)? {
        let entry = entry?;
        let src = entry.path();

        println!("HEREx");
        let extension = Path::new(src.as_os_str())
            .extension()
            .and_then(|os_str| os_str.to_str());
        let new_path = format!("{}/{}.{}", dest_path.to_string_lossy(), theme_name, extension.unwrap_or(""));
        println!("{}", &new_path);
        std::fs::copy(entry.path(), new_path)?;

    }

    Ok(())
}

