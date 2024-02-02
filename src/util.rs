pub mod util {
    use std::path::{Component, Path, PathBuf};

    pub trait IsOlder {
        fn is_older(&self, other: &PathBuf) -> Result<bool, std::io::Error>;
    }

    impl IsOlder for PathBuf {
        fn is_older(&self, other: &PathBuf) -> Result<bool, std::io::Error> {
            if self.metadata()?.modified()? < other.metadata()?.modified()? {
                return Ok(true);
            }
            Ok(false)
        }
    }

    pub fn normalize_path(path: &Path) -> PathBuf {
        let mut components = path.components().peekable();
        let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
            components.next();
            PathBuf::from(c.as_os_str())
        } else {
            PathBuf::new()
        };

        for component in components {
            match component {
                Component::Prefix(..) => unreachable!(),
                Component::RootDir => {
                    ret.push(component.as_os_str());
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    ret.pop();
                }
                Component::Normal(c) => {
                    ret.push(c);
                }
            }
        }
        ret
    }

    pub fn compose_path<P: Into<PathBuf>>(dir: P, file: P) -> PathBuf {
        let mut dir = Into::<PathBuf>::into(dir).clone();
        dir.push(file.into());
        return normalize_path(&dir);
        /*let path = format!(
            "{}{}{}",
            dir.into().to_str().unwrap(),
            MAIN_SEPARATOR
            file.into().to_str().unwrap(),
        );
        return PathBuf::from_str(&path).unwrap();*/
    }
}
