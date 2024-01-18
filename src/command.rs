pub mod Command {
    use crate::util::util::{compose_path, IsOlder};
    use crate::ProjType;
    use libc::system;
    use path_absolutize::Absolutize;
    use path_slash::PathExt;
    use serde::Deserialize;
    use std::{
        collections::HashMap,
        fs::{metadata, read_dir, remove_file, File, ReadDir},
        io::{Read, Write},
        path::{Path, PathBuf, MAIN_SEPARATOR},
    };

    #[cfg(target_os = "windows")]
    static SYSYEM_LIBS: &str = "-lshell32 -ladvapi32 -lcfgmgr32 -lcomctl32 -lcomdlg32 -ld2d1 -ldwrite -ldxgi -lgdi32 -lkernel32 -lmsimg32 -lole32 -lopengl32 -lshlwapi -luser32 -lwindowscodecs -lwinspool -luserenv -lws2_32 -lbcrypt -lmsvcrt -loleaut32 -luuid -lodbc32 -lodbccp32";
    #[cfg(target_os = "linux")]
    static SYSYEM_LIBS: &str = "";

    static BUILD_FILE: &str = "./build.cpp";

    #[derive(Debug)]
    pub struct Command {
        dir: PathBuf,
        compiler: String,
        linker: String,
        linker_flags: Vec<String>,
        compiler_flags: Vec<String>,
        out_dir: String,
        src: Vec<PathBuf>,
        name: String,
        typ: ProjType,
        gen_config: bool,
        build_file: bool,
        includes: Vec<PathBuf>,
    }

    #[derive(Deserialize)]
    struct Config {
        package: Option<Package>,
        compiler: Option<Compiler>,
        linker: Option<Linker>,
        dependencies: Option<HashMap<String, Dependency>>,
        lib: Option<Lib>,
    }

    #[derive(Debug, Deserialize)]
    struct Dependency {
        path: Option<PathBuf>,
        git: Option<String>,
        version: Option<String>,
        leaky: Option<bool>,
    }

    #[derive(Deserialize)]
    struct Lib {
        header: Vec<PathBuf>,
    }

    #[derive(Deserialize)]
    struct Package {
        name: Option<String>,
        outdir: Option<String>,
        src: Option<Vec<String>>,
        typ: Option<String>,
        gen_config: Option<bool>,
        collect_rec: Option<bool>,
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
        fn new() -> Command {
            let mut command = Command {
                dir: PathBuf::new(),
                compiler: String::from("clang"),
                linker: String::from("clang"),
                linker_flags: Vec::new(),
                compiler_flags: Vec::new(),
                out_dir: String::from("."),
                src: Vec::new(),
                name: String::from("a"),
                typ: ProjType::bin,
                gen_config: false,
                build_file: false,
                includes: Vec::new(),
            };

            command.src.push(PathBuf::from("src/main.c"));

            return command;
        }

        fn get_includes(&self) -> Vec<String> {
            let mut includes = Vec::new();
            self.compiler_flags.iter().for_each(|f| {
                if f.starts_with("-I") {
                    includes.push(f.clone());
                }
            });
            return includes;
        }

        fn get_compile_command_for_file(&self, file: &mut PathBuf) -> String {
            let mut cmd = String::new();
            cmd.push_str(self.compiler.as_str());
            //println!("{:?}", file);
            cmd.push_str(format!(" -c {}", file.to_str().unwrap()).as_str());
            file.set_extension("o");
            cmd.push_str(format!(" -o {}", file.to_str().unwrap()).as_str());
            for flag in &self.compiler_flags {
                cmd.push_str(format!(" {}", flag).as_str());
            }
            return cmd;
        }

        pub fn compile(&mut self) {
            let obj: Vec<PathBuf> = self
                .src
                .iter()
                .map(|item| {
                    let mut item = item.clone();
                    item.set_extension("o");
                    return item;
                })
                .collect();
            for (i, file) in self.src.iter().enumerate() {
                if obj[i].exists() {
                    if file.is_older(&obj[i]).unwrap() {
                        continue;
                    }
                }
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
            match self.typ {
                ProjType::bin => cmd.push_str(self.linker.as_str()),
                ProjType::dynlib => {
                    cmd.push_str(&self.linker.as_str());
                    self.linker_flags.insert(0, format!("-shared"));
                }
                ProjType::staticlib => cmd.push_str("ar -rcs"),
            }

            if self.typ != ProjType::staticlib {
                cmd.push_str(format!(" -o {}", self.get_out_file()).as_str());
            } else {
                cmd.push_str(format!(" {}", self.get_out_file()).as_str());
            }

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

                    let path = self.dir.clone();
                    let mut buf = file.clone();
                    buf.set_extension("o");
                    entries.push(format!(
                    "{{ \"directory\": \"{}\", \"file\": \"{}\", \"output\": \"{}\", \"arguments\": [\"{}\", {}] }}",
                    path.absolutize()
                        .unwrap()
                        .to_slash()
                        .unwrap(),
                    file.to_slash().unwrap(),
                    buf.to_slash().unwrap(),
                    self.compiler.as_str(),
                    args
                ));
                }

                let path = compose_path(&self.dir, &PathBuf::from("compile_commands.json"));
                match File::create(&path) {
                    Ok(mut fd) => {
                        fd.write("[\n".as_bytes()).expect(
                            format!("Could not wirte to file `{}`", path.to_str().unwrap())
                                .as_str(),
                        );

                        for i in 0..entries.len() {
                            if i != entries.len() - 1 {
                                entries[i].push(',');
                            }
                            fd.write(entries[i].as_bytes()).expect(
                                format!("Could not wirte to file `{}`", path.to_str().unwrap())
                                    .as_str(),
                            );
                        }

                        fd.write("\n]".as_bytes()).expect(
                            format!("Could not wirte to file `{}`", path.to_str().unwrap())
                                .as_str(),
                        );
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

            if self.typ == ProjType::dynlib || self.typ == ProjType::staticlib {
                return;
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

            let exe = format!(
                "{}{}{}.{}",
                self.out_dir, MAIN_SEPARATOR, self.name, file_ext
            );

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

        fn should_rebuild(&self) -> bool {
            if !PathBuf::from(self.get_out_file()).exists() {
                return true;
            }
            for file in &self.src {
                if PathBuf::from(self.get_out_file()).is_older(file).unwrap() {
                    return true;
                }
            }
            false
        }

        pub fn get_out_file(&self) -> String {
            return format!(
                "{}{}{}.{}",
                compose_path(&self.dir, &PathBuf::from(&self.out_dir))
                    .to_str()
                    .unwrap(),
                MAIN_SEPARATOR,
                self.name,
                self.typ.get_file_ext()
            );
        }

        fn resolve_dependency(&mut self, dep: &Dependency) {
            if dep.git.is_some() && dep.path.is_some() {
                println!("Cannot use path and git at the same time");
                std::process::exit(1);
            }
            if dep.git.is_none() && dep.path.is_none() {
                println!("Need either path or git in dependency");
                std::process::exit(1);
            }

            if let Some(path) = &dep.path {
                let cfg_path = compose_path(path, &PathBuf::from("Cargoc.toml"));
                if !cfg_path.exists() {
                    println!(
                        "{}",
                        format!(
                            "Invalid path to cargoc.toml `{}`",
                            cfg_path.to_str().unwrap()
                        )
                    );
                    std::process::exit(1);
                }
                let mut cmd = get(cfg_path.to_str().unwrap());

                if cmd.typ == ProjType::bin || cmd.includes.len() < 1 {
                    println!("Cannot use non library project as a dependency and or need includes");
                    std::process::exit(1);
                }
                if cmd.should_rebuild() {
                    cmd.build();
                }

                if dep.leaky.unwrap_or(false) {
                    let mut incls: Vec<PathBuf> = cmd.includes.iter().map(|i| compose_path(path, i)).collect();
                    self.includes.append(&mut incls);
                }

                //println!("includes -> {:#?}", cmd.includes);
                for include in &cmd.includes {
                    self.compiler_flags.push(format!(
                        "-I{}",
                        compose_path(path, include).to_slash().unwrap()
                    ));
                }
                self.linker_flags.push(cmd.get_out_file());
            }
        }
    }

    fn dir_collect_files(dir: ReadDir, descend: bool) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut src_files = Vec::<PathBuf>::new();
        for file in dir {
            let file = file?.path();
            if file.is_file() {
                match file.extension().unwrap().to_str().unwrap() {
                    "c" | "cpp" | "cxx" | "c++" | "ly" => {
                        src_files.push(file);
                    }
                    _ => {}
                }
            } else if descend {
                let sub_dir = read_dir(file).unwrap();
                src_files.append(&mut dir_collect_files(sub_dir, descend)?);
            }
        }

        return Ok(src_files);
    }

    pub fn get(file: &str) -> Command {
        let mut config_file: File = File::open(file).expect("Could not open file");

        let mut contents = String::new();
        config_file.read_to_string(&mut contents).unwrap();

        let config: Config =
            toml::from_str(contents.as_str()).expect("Could not process toml file");

        let mut command: Command = Command::new();

        let dir = Path::new(file).parent();

        if let Some(package) = &config.package {
            if let Some(dir) = dir {
                command.dir = dir.to_path_buf();
            }

            if let Some(typ) = &package.typ {
                command.typ = ProjType::get(typ.clone());
            }

            if let Some(name) = &package.name {
                command.name = name.clone();
            }

            if let Some(dir) = &package.outdir {
                command.out_dir = dir.clone();
            }

            if let Some(src) = &package.src {
                command.src.pop();
                for file in src {
                    let path = compose_path(&command.dir, &PathBuf::from(file));
                    match path.metadata() {
                        Ok(md) => {
                            if md.is_file() {
                                match path.extension().unwrap().to_str().unwrap() {
                                    "c" | "cpp" | "cxx" | "c++" | "ly" => {
                                        command.src.push(path.to_path_buf());
                                    }
                                    _ => {}
                                }
                            } else {
                                let dir = read_dir(path).unwrap();
                                command.src.append(
                                    &mut dir_collect_files(
                                        dir,
                                        package.collect_rec.unwrap_or(false),
                                    )
                                    .expect("Could not read dir"),
                                );
                            }
                        }
                        Err(_) => {
                            println!("Could not find the path specified['{}']", file);
                            std::process::exit(1);
                        }
                    }
                }
            }
            if let Some(cfg) = package.gen_config {
                command.gen_config = cfg;
            }
        }

        if let Some(compiler) = &config.compiler {
            if let Some(compiler) = &compiler.compiler {
                command.compiler = compiler.clone();
            }
            if let Some(flags) = &compiler.flags {
                for flag in flags {
                    command.compiler_flags.push(flag.clone());
                }
            }
            if let Some(includes) = &compiler.includes {
                for include in includes {
                    command.compiler_flags.push(format!(
                        "-I{}",
                        compose_path(&command.dir, &PathBuf::from(include))
                            .to_str()
                            .unwrap()
                    ));
                }
            }
        }

        if let Some(linker) = &config.linker {
            if let Some(linker) = &linker.linker {
                command.linker = linker.clone();
            }
            if let Some(flags) = &linker.flags {
                for flag in flags {
                    command.linker_flags.push(flag.clone());
                }
            }
            if let Some(libs) = &linker.libs {
                for lib in libs {
                    command.linker_flags.push(
                        compose_path(&command.dir, &PathBuf::from(lib))
                            .to_str()
                            .unwrap()
                            .to_string(),
                    );
                }
            }
            if let Some(v) = linker.default_libs {
                if v {
                    command.linker_flags.append(
                        &mut SYSYEM_LIBS
                            .split_whitespace()
                            .map(|item| item.to_string())
                            .collect(),
                    );
                }
            }
        }

        if let Some(dependencies) = &config.dependencies {
            for (name, content) in dependencies {
                //println!("name -> {name}");
                command.resolve_dependency(content);
            }
        }

        if let Some(lib) = &config.lib {
            command.includes = lib.header.clone();
        }

        if Path::new(BUILD_FILE).exists() {
            command.build_file = true;
        }
        return command;
    }
}
