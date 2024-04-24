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
            dbg!(&components);
            components.next();
            PathBuf::from(c.as_os_str())
        } else {
            PathBuf::new()
        };

        for (i, component) in components.enumerate() {
            match component {
                Component::Prefix(..) => unreachable!(),
                Component::RootDir => {
                    ret.push(component.as_os_str());
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    if i != 0 {
                        ret.pop();
                    } else {
                        ret.push("..");
                    }
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
        let file = Into::<PathBuf>::into(file);
        if dir.components().next().is_some() {
            dir.push(file);
            return normalize_path(&dir);
        }
        return normalize_path(&file);
        /*let path = format!(
            "{}{}{}",
            dir.into().to_str().unwrap(),
            MAIN_SEPARATOR
            file.into().to_str().unwrap(),
        );
        return PathBuf::from_str(&path).unwrap();*/
    }
}
