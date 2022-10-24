use clap::{arg, command, Parser};
use command::Command;

use std::{
    fs::{self, metadata, DirBuilder, File},
    io::Write,
    path::PathBuf,
};
#[allow(non_snake_case)]
mod command;

#[allow(non_camel_case_types)]
#[derive(Debug)]
enum ProjType {
    bin = 0,
    dynlib = 1,
    staticlib = 2,
}

impl ProjType {
    pub fn get(str: String) -> ProjType {
        match str.as_str() {
            "bin" => {
                return ProjType::bin;
            }
            "dynlib" => {
                return ProjType::dynlib;
            }
            "staticlib" => {
                return ProjType::staticlib;
            }
            x => {
                println!("Unknown project type '{}'", x);
                std::process::exit(1);
            }
        }
    }

    pub fn get_file_ext(&self) -> &str {
        match self {
            ProjType::bin => return "exe",
            ProjType::dynlib => return "dll",
            ProjType::staticlib => return "lib",
        }
    }
}

fn try_resolve_dependency(path: PathBuf) {
    match metadata(&path) {
        Ok(fd) => {
            if fd.is_file() {
                println!("test");
                return;
            }

            let path_as_string = path.as_path().to_str().unwrap();
            let manifest_path = format!("{}/Cargoc.toml", path_as_string);
            match metadata(&manifest_path) {
                Ok(_) => {
                    let cmd = Command::get(&manifest_path);
                    println!("{:?}", cmd);
                    println!("path -> {}", path_as_string);
                    println!("out -> {}", cmd.get_out_file());
                    let lib_file = format!("{}{}", path_as_string, cmd.get_out_file());
                    println!("lib -> {}", lib_file);
                    match metadata(&lib_file) {
                        Ok(_) => {}
                        Err(_) => {
                            //build dependency
                        }
                    }
                }
                Err(_) => {
                    println!(
                        "Cannot resolve dependency '{}'; Cargoc.toml file not found",
                        path_as_string
                    );
                    std::process::exit(1);
                }
            }
        }
        Err(_) => {
            println!(
                "Could not find the path specified['{}']",
                path.as_path().to_str().unwrap()
            );
            std::process::exit(1);
        }
    }
}

fn init_proj(name: &str, _bin: bool, lib: bool) {
    let path = format!("./{}", name);

    //create project dir
    match fs::DirBuilder::new().create(path.as_str()) {
        Ok(_) => {}
        Err(_) => {
            println!("Could not create dir['{}']", path);
            std::process::exit(1);
        }
    }

    //create default proj files
    //cargoc.toml
    match File::create(format!("{}/Cargoc.toml", path).as_str()) {
        Ok(mut fd) => {
            let mut typ = "bin";

            if lib {
                typ = "dynlib";
            }

            let file_content = format!(
                "[package]
name = \"{}\"
typ = \"{}\"",
                name, typ
            );

            match fd.write_all(file_content.as_str().as_bytes()) {
                Ok(_) => {}
                Err(_) => {
                    println!("Could not wirte to file['{}/Cargoc.toml']", path);
                    std::process::exit(1);
                }
            }
        }
        Err(_) => {
            println!("Could not create file['{}/Cargoc.toml']", path);
            std::process::exit(1);
        }
    }

    //create proj dir
    match DirBuilder::new().create(format!("{}/src", path).as_str()) {
        Ok(_) => {}
        Err(_) => {
            println!("Could not create dir['{}/src']", path);
            std::process::exit(1);
        }
    }

    match File::create(format!("{}/src/main.c", path).as_str()) {
        Ok(mut fd) => {
            match fd.write_all("int main(int argc, char **argv) {\n\treturn 0;\n}".as_bytes()) {
                Ok(_) => {}
                Err(_) => {
                    println!("Could not write to file['{}/src/main.c']", path);
                    std::process::exit(1);
                }
            }
        }
        Err(_) => {
            println!("Could not create file['{}/src/main.c']", path);
            std::process::exit(1);
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    command: Option<String>,
    name: Option<String>,
    /// Specify binary target
    #[arg(long)]
    bin: bool,
    /// Specify library target
    #[arg(long)]
    lib: bool,
}

fn main() {
    let args = Args::parse();
    if args.command.is_some() {
        match args.command.unwrap().as_str() {
            "run" => Command::get("Cargoc.toml").run(),
            "build" => Command::get("Cargoc.toml").build(),
            "init" => init_proj(
                args.name.unwrap_or(String::new()).as_str(),
                args.bin,
                args.lib,
            ),
            "clean" => Command::get("Cargoc.toml").clean(),
            x => {
                println!("unknown command '{}'", x);
            }
        }
    }

    /*
    let cd: std::ffi::CString = std::ffi::CString::new("cd test & ls -l").unwrap();
    let ls = std::ffi::CString::new("ls -l").unwrap();
    unsafe {
        system(cd.as_ptr());
        system(ls.as_ptr());
    }
    */
}
