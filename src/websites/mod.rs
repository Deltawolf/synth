use serde::{Deserialize, Serialize};
use std::error::Error;
use toml::Table;
use crate::InputArgs;
use crate::errors::NoResultError;

#[derive(Debug,Deserialize, Serialize)]
pub struct Wallbase {
    data: Vec<WallbaseData>
}
#[derive(Debug,Deserialize, Serialize)]
struct WallbaseData{
    path: String,
    resolution: String,
    views: i32,
    favorites: i32,
    id: String,
}

pub trait Website {

    fn build_url(file_path: String, inputargs: InputArgs) -> Result<String, Box<dyn Error>>;
    fn build(file_path: String, inputargs: InputArgs) -> Result<Wallbase, Box<dyn Error>>;
    fn get_image(&self) -> String;
}

impl Website for Wallbase {

    fn build(file_path: String, inputargs: InputArgs) -> Result<Wallbase, Box<dyn Error>> {

        let res = Wallbase::build_url(file_path, inputargs);
        if res.is_err() {
            return Err(res.unwrap_err())
        }

        let url = res.unwrap();
        let response = reqwest::blocking::get(url)?.text()?;
        let wallbase: Wallbase = serde_json::from_str(response.as_str())?;

        if wallbase.data.is_empty() {
            return Err(Box::new(NoResultError::new("No results found")));
        }
        Ok(wallbase)
    }

    fn build_url(file_path: String, inputargs: InputArgs) -> Result<String, Box<dyn Error>> {

        let contents = std::fs::read_to_string(file_path)?;
        let config = contents.parse::<Table>().unwrap();

        let url_parts = vec![
            config["base_url"].as_str().unwrap(), 
            config["api"].as_str().unwrap(),
            "&categories=",
            config["categories"].as_str().unwrap(),
            "&purity=",
            config["purity"].as_str().unwrap(),
            "&sorting=random"];
        
            let mut url = url_parts.join("");

        if inputargs.color.is_some() {
            url.push_str("&color=");
            url.push_str(inputargs.color.unwrap().as_str());
        }

        if inputargs.query.is_some() {
            url.push_str("&q=");
            let query = inputargs.query.unwrap().as_str().replace(" ", "+");
            url.push_str(query.as_str());
        }

        if inputargs.resolution.is_some() {
            url.push_str("&resolutions=");
            url.push_str(inputargs.resolution.unwrap().as_str());

        }

        Ok(url)
    
    }

    fn get_image(&self) -> String {
        self.data[0].path.to_owned()
    }

}


