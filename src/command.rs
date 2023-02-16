pub mod Command {
    use crate::ProjType;
    use libc::system;
    use path_absolutize::Absolutize;
    use path_slash::PathExt;
    use serde::Deserialize;
    use std::{
        collections::HashMap,
        fs::{metadata, read_dir, remove_file, File},
        io::{Read, Write},
        path::{Path, PathBuf, MAIN_SEPARATOR},
    };

    static SYSYEM_LIBS: &str = "-lshell32 -ladvapi32 -lcfgmgr32 -lcomctl32 -lcomdlg32 -ld2d1 -ldwrite -ldxgi.lgdi32 -lkernel32 -lmsimg32 -lole32 -lopengl32 -lshlwapi -luser32 -lwindowscodecs -lwinspool -luserenv -lws2_32 -lbcrypt -lmsvcrt -loleaut32 -luuid -lodbc32 -lodbccp32";

    #[derive(Debug)]
    pub struct Command {
        compiler: String,
        linker: String,
        linker_flags: Vec<String>,
        compiler_flags: Vec<String>,
        out_dir: String,
        src: Vec<String>,
        name: String,
        typ: ProjType,
        gen_config: bool,
    }

    #[derive(Deserialize)]
    struct Config {
        package: Option<Package>,
        compiler: Option<Compiler>,
        linker: Option<Linker>,
        dependencies: Option<HashMap<String, String>>,
    }

    #[derive(Deserialize)]
    struct Package {
        name: Option<String>,
        outdir: Option<String>,
        src: Option<Vec<String>>,
        typ: Option<String>,
        gen_config: Option<bool>,
    }

    #[derive(Deserialize)]
    struct Compiler {
        includes: Option<Vec<String>>,
        flags: Option<Vec<String>>,
        compiler: Option<String>,
    }

    #[derive(Deserialize)]
    struct Linker {
        libs: Option<Vec<String>>,
        flags: Option<Vec<String>>,
        linker: Option<String>,
        default_libs: Option<bool>,
    }

    impl Command {
        pub fn new() -> Command {
            let mut command = Command {
                compiler: String::from("clang"),
                linker: String::from("clang"),
                linker_flags: Vec::new(),
                compiler_flags: Vec::new(),
                out_dir: String::from("."),
                src: Vec::new(),
                name: String::from("a"),
                typ: ProjType::bin,
                gen_config: false,
            };

            command.src.push(String::from("src/main.c"));

            return command;
        }

        fn get_compile_command_for_file(&self, file: &mut PathBuf) -> String {
            let mut cmd = String::new();
            cmd.push_str(self.compiler.as_str());
            cmd.push_str(format!(" -c {}", file.to_str().unwrap()).as_str());
            file.set_extension("o");
            cmd.push_str(format!(" -o {}", file.to_str().unwrap()).as_str());
            for flag in &self.compiler_flags {
                cmd.push_str(format!(" {}", flag).as_str());
            }
            return cmd;
        }

        pub fn compile(&mut self) {
            for file in &self.src {
                let cmd = std::ffi::CString::new(
                    self.get_compile_command_for_file(&mut PathBuf::from(file)),
                )
                .unwrap();
                println!("[compiling] -> {}", cmd.to_str().unwrap());
                if unsafe { system(cmd.as_ptr()) } != 0 {
                    std::process::exit(1);
                }
            }
        }

        pub fn get_linker_command(&mut self) -> String {
            let mut cmd: String = String::new();
            let file_ext = self.typ.get_file_ext();
            match self.typ {
                ProjType::bin => cmd.push_str(self.linker.as_str()),
                ProjType::dynlib => {
                    cmd.push_str(&self.linker.as_str());
                    self.linker_flags.push(format!("-shared"));
                }
                ProjType::staticlib => cmd.push_str("ar -rcs"),
            }

            cmd.push_str(format!(" -o {}/{}.{}", self.out_dir, self.name, file_ext).as_str());

            for src in &self.src {
                let mut file_path = PathBuf::from(src);
                file_path.set_extension("o");
                cmd.push_str(format!(" {}", file_path.to_str().unwrap()).as_str());
            }

            for flag in &self.linker_flags {
                cmd.push_str(format!(" {}", flag).as_str());
            }

            return cmd;
        }

        pub fn link(&mut self) {
            let cmd = std::ffi::CString::new(self.get_linker_command()).unwrap();
            println!("[linking] -> {}", cmd.to_str().unwrap());
            if unsafe { system(cmd.as_ptr()) } != 0 {
                std::process::exit(1);
            }
        }

        pub fn try_create_config(&mut self) {
            if self.gen_config {
                let mut entries: Vec<String> = Vec::new();
                for file in &self.src {
                    let mut args: String = String::new();

                    for i in 0..self.compiler_flags.len() {
                        if i == self.compiler_flags.len() - 1 {
                            args.push_str(
                                format!("\"{}\"", self.compiler_flags[i].as_str()).as_str(),
                            );
                            continue;
                        }
                        args.push_str(
                            format!("\"{}\", ", self.compiler_flags[i].as_str()).as_str(),
                        );
                    }

                    let path = Path::new(".");
                    let mut buf = PathBuf::from(file.as_str());
                    buf.set_extension("o");
                    entries.push(format!(
                    "{{ \"directory\": \"{}\", \"file\": \"{}\", \"output\": \"{}\", \"arguments\": [\"{}\", {}] }}",
                    path.absolutize()
                        .unwrap()
                        .to_slash()
                        .unwrap()
                        .to_string()
                        .as_str(),
                    file.as_str(),
                    buf.to_str().unwrap(),
                    self.compiler.as_str(),
                    args
                ));
                }

                let path = format!("compile_commands.json");
                match File::create(path) {
                    Ok(mut fd) => {
                        fd.write("[\n".as_bytes());

                        for i in 0..entries.len() {
                            if i != entries.len() - 1 {
                                entries[i].push(',');
                            }
                            fd.write(entries[i].as_bytes());
                        }

                        fd.write("\n]".as_bytes());
                    }
                    Err(e) => {
                        println!("Could not open file['{}']", e);
                        std::process::exit(1);
                    }
                }
            }
        }

        pub fn build(&mut self) {
            self.compile();
            self.link();
            self.try_create_config();
        }

        pub fn run(&mut self) {
            self.build();

            match self.typ {
                ProjType::bin => {}
                ProjType::dynlib | ProjType::staticlib => return,
            }

            let programm = std::ffi::CString::new(
                format!("{}{}{}.exe", self.out_dir, MAIN_SEPARATOR, self.name).as_str(),
            )
            .unwrap();
            println!("[running] -> {}", programm.to_str().unwrap());
            if unsafe { system(programm.as_ptr()) } != 0 {
                std::process::exit(1);
            }
        }

        pub fn clean(&self) {
            for src in &self.src {
                let mut obj = PathBuf::from(src);
                obj.set_extension("o");
                match remove_file(obj.to_str().unwrap()) {
                    Ok(_) => {
                        println!("[Clean] -> {}", obj.to_str().unwrap());
                    }
                    Err(_) => {
                        println!("Could not delete file['{}']", obj.to_str().unwrap());
                        std::process::exit(1);
                    }
                }
            }

            let file_ext = self.typ.get_file_ext();

            let mut exe = String::new();
            if cfg!(windows) {
                exe = format!("{}\\{}.{}", self.out_dir, self.name, file_ext);
            } else if cfg!(unix) {
                exe = format!("./{}/{}.{}", self.out_dir, self.name, file_ext);
            }

            match remove_file(exe.as_str()) {
                Ok(_) => {
                    println!("[Clean] -> {}", exe);
                }
                Err(_) => {
                    println!("Could not delete file['{}']", exe);
                    std::process::exit(1);
                }
            }
        }

        pub fn get_out_file(&self) -> String {
            return format!(
                "{}{}{}.{}",
                self.out_dir,
                MAIN_SEPARATOR,
                self.name,
                self.typ.get_file_ext()
            );
        }
    }

    pub fn get(file: &str) -> Command {
        let mut config_file: File = File::open(file).expect("Could not open file");

        let mut contents = String::new();
        config_file.read_to_string(&mut contents).unwrap();

        let config: Config =
            toml::from_str(contents.as_str()).expect("Could not process toml file");

        let mut command: Command = Command::new();

        if config.package.is_some() {
            let package: Package = config.package.unwrap();
            match package.typ {
                Some(v) => {
                    command.typ = ProjType::get(v);
                }
                None => {}
            }

            match package.name {
                Some(v) => {
                    command.name = v;
                }
                None => {}
            }
            match package.outdir {
                Some(v) => {
                    command.out_dir = v;
                }
                None => {}
            }
            match package.src {
                Some(v) => {
                    command.src.pop();
                    for file in v {
                        //check for dir
                        //let md = metadata(&file).unwrap();
                        match metadata(&file) {
                            Ok(md) => {
                                if md.is_file() {
                                    let path = Path::new(&file);
                                    match path.extension().unwrap().to_str().unwrap() {
                                        "c" | "cpp" | "cxx" | "c++" => {
                                            command.src.push(file.to_string());
                                        }
                                        "h" | "hpp" | "o" | "obj" | "exe" | "out" => {}
                                        x => {
                                            panic!("unknown file type -> '{}'", x)
                                        }
                                    }
                                } else {
                                    let files = read_dir(file).unwrap();

                                    for file in files {
                                        let name = file.unwrap().path().display().to_string();
                                        let path = Path::new(&name);
                                        if path.is_dir() {
                                            //maybe traverse path recursevely
                                            continue;
                                        }
                                        match path.extension().unwrap().to_str().unwrap() {
                                            "c" | "cpp" | "cxx" | "c++" => {
                                                command.src.push(name);
                                            }
                                            "h" | "hpp" | "o" | "obj" | "exe" | "out" => {}
                                            x => {
                                                panic!("unknown file type -> '{}'", x)
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                println!("Could not find the path specified['{}']", file);
                                std::process::exit(1);
                            }
                        }
                    }
                }
                None => {}
            }
            match package.gen_config {
                Some(v) => {
                    command.gen_config = v;
                }
                None => {}
            }
        }

        if config.compiler.is_some() {
            let compiler: Compiler = config.compiler.unwrap();

            match compiler.compiler {
                Some(v) => {
                    command.compiler = v;
                }
                None => {}
            }
            match compiler.flags {
                Some(v) => {
                    for flag in v {
                        command.compiler_flags.push(flag);
                    }
                }
                None => {}
            }
            match compiler.includes {
                Some(v) => {
                    for include in v {
                        command.compiler_flags.push(format!("-I{}", include));
                    }
                }
                None => {}
            }
        }

        if config.linker.is_some() {
            let linker: Linker = config.linker.unwrap();
            match linker.linker {
                Some(v) => command.linker = v,
                None => {}
            }
            match linker.flags {
                Some(v) => {
                    for flag in v {
                        command.linker_flags.push(flag);
                    }
                }
                None => {}
            }
            match linker.libs {
                Some(v) => command.linker_flags = v,
                None => {}
            }
            if let Some(v) = linker.default_libs {}
        }

        if config.dependencies.is_some() {
            let dependencies = config.dependencies.unwrap();
            for value in dependencies {
                //println!("value -> {}", value.1);
                crate::try_resolve_dependency(PathBuf::from(value.1));
            }
        }
        return command;
    }
}
