pub mod util {
    use std::{
        path::{Path, PathBuf, MAIN_SEPARATOR},
        str::FromStr,
    };
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

    pub fn compose_path<P: Into<PathBuf>>(dir: P, file: P) -> PathBuf {
        let mut dir = Into::<PathBuf>::into(dir).clone();
        dir.push(file.into());
        return dir;
        /*let path = format!(
            "{}{}{}",
            dir.into().to_str().unwrap(),
            MAIN_SEPARATOR,
            file.into().to_str().unwrap(),
        );
        return PathBuf::from_str(&path).unwrap();*/
    }
}
