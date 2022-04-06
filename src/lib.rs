use std::error::Error;  //allows for some better errors
use std::fs::{self}; //the library that will allow us to parse files
use std::path::{Path, PathBuf};    //the library that will allow us to get more info about files and directories      
use std::ffi::OsString;

use id3::{Tag, TagLike, Version}; //library that allows us to access and modify the tags of media files


const SUPPORTED_EXTENSIONS: [&str; 2] = [
    "mp3",
    "wav"
];

pub struct Config {
    pub paths: Vec<PathBuf>,
}
impl Config {
    pub fn new(args: &[String]) -> Result<Config, Box<dyn Error>> {
        //DATA
        let mut config: Config = Config {
            paths: Vec::new(),
        };
        
        //ensure enough arguments were given
        if args.len() <= 1 {
            return Err("No Paths Given".into());
        }

        //parse args
        let raw_paths: Vec<String> = (&args[0..]).iter().map(|s| s.clone()).collect(); //skip the first one


        //make sure given paths are all valid locations, and add those paths to config.paths as needed
        for path in raw_paths.iter() {
            if !Path::new(path).exists() {
                return Err( format!("Invalid path(s) given\n\t{}\n\tdoes not exist", path).into() );
            }
        }

        //add paths (including those to sub-files) from raw_paths to config.paths based
        raw_paths.iter().map(|p| Path::new(&p).to_path_buf()).for_each(|path| {
            if path.is_file() { //if it's a file
                config.paths.push(path);
            } 
            else if path.is_dir() { //if it's a directory
                config.paths.append(&mut list_files(&path));
            }
        });

        //filter out paths to files that don't have a supported extension
        config.paths = config.paths.into_iter().filter(|path| {
            if let Some(extension) = path.extension() {
                if SUPPORTED_EXTENSIONS.iter().map(|ext| OsString::from(ext)).any(|ext| ext.eq_ignore_ascii_case(extension)) {
                    return true;
                }
            }
            return false;
        }).collect();

        //return
        Ok(config)
    }

}


/**
 * run the program
 */
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    //go through every path
    for path in config.paths.iter() {
        //read tags
        let mut tag = match Tag::read_from_path(path) {
            Ok(tag) => tag,
            Err(err) => {
                eprintln!("{}", err);
                continue;
            },
        };

        //if we can, read an artist
        let artist = match tag.artist() {
            Some(a) => a.to_string(),
            None => {
                eprintln!("error reading value for tag: Artist");
                continue;
            },
        };

        let album_artist = artist.split(',').next().unwrap_or_else(|| "").to_string();
        println!("{:?}\n\tAlbum Artist set to: {}", path.as_os_str(), album_artist);

        //write album artist
        tag.set_album_artist(album_artist);


        //write new data to file at path
        tag.write_to_path(path, Version::Id3v24)?;
    }

    return Ok(());
}








/**
 * returns a vector containing paths to all files in path and subdirectories of path
 */
fn list_files(path: &Path) -> Vec<PathBuf> {
    let mut vec = Vec::new();
    _list_files(&mut vec,&path);
    vec
}
fn _list_files(vec: &mut Vec<PathBuf>, path: &Path) {
    if path.is_dir() {
        let paths = fs::read_dir(&path).unwrap();
        for path_result in paths {
            let full_path = path_result.unwrap().path();
            if full_path.is_dir() {
                _list_files(vec, &full_path);
            } else {
                vec.push(full_path);
            }
        }
    }
}

pub fn help() {
    println!("                              update-album-artist.exe");
    println!("                                 By Anthony Rubick\n");
    println!("given a the path to a folder or file, updates the metadata of every music file (see accepted file formats below) to include the album artist (the first artist to appear in the comma separated list of artists in the metadata).\n");

    println!("USAGE:\n\tupdate-metadata-album artist [PATHS]\n");
    
    println!("PATH:\n\tPath to the file (or folder of files) to update\nif you pass multiple arguments, it assume they are more PATHS to update\n");
    
    println!("ACCEPTED FILE FORMATS:\n\tmp3\n\twav");
    println!();
}