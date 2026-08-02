use ignore::WalkBuilder;
use is_executable::IsExecutable;
use std::{
    collections::HashMap,
    fs::{OpenOptions, read_to_string},
    io::{BufWriter, Write},
    path::Path,
};

#[derive(Debug)]
/// A struct representing the Qwx file system, which provides methods for interacting with files and directories.
pub struct QwxFileSystem {
    pub files: Vec<String>,
    pub dirs: Vec<String>,
    pub symlinks: Vec<String>,
    pub executables: Vec<String>,
    pub loaded: HashMap<String, String>,
}

/// All ignores files supported
pub const IGNORE_FILES: [&str; 4] = [".awqignore", ".gitignore", ".hgignore", ".dockerignore"];

impl QwxFileSystem {
    #[must_use]
    pub fn new<P: AsRef<Path>>(p: &P) -> Self {
        let mut w = WalkBuilder::new(p);
        w.threads(num_cpus::get());

        for i in &IGNORE_FILES {
            w.add_custom_ignore_filename(i);
        }
        let end = w.build();
        let mut data = Self {
            files: Vec::new(),
            dirs: Vec::new(),
            symlinks: Vec::new(),
            executables: Vec::new(),
            loaded: HashMap::new(),
        };
        for f in end.flatten() {
            if f.path().is_file() {
                data.files.push(
                    f.path()
                        .to_str()
                        .expect("failed to convert path")
                        .to_string()
                        .replace("./", ""),
                );
            } else if f.path().is_dir() {
                data.dirs.push(
                    f.path()
                        .to_str()
                        .expect("failed to convert path")
                        .to_string()
                        .replace("./", ""),
                );
            } else if f.path().is_symlink() {
                data.symlinks.push(
                    f.path()
                        .to_str()
                        .expect("failed to convert path")
                        .to_string()
                        .replace("./", ""),
                );
            } else if f.path().is_executable() {
                data.executables.push(
                    f.path()
                        .to_str()
                        .expect("failed to convert path")
                        .to_string()
                        .replace("./", ""),
                );
            }
        }
        data
    }
    /// get all dirs in the given path
    pub fn get_dirs(&self) -> Vec<String> {
        self.dirs.to_vec()
    }
    /// get all symlinks in the given path
    pub fn get_symlinks(&self) -> Vec<String> {
        self.symlinks.to_vec()
    }
    /// get all files in the given path
    pub fn get_files(&self) -> Vec<String> {
        self.files.to_vec()
    }
    /// get all executables in the given path
    pub fn get_executables(&self) -> Vec<String> {
        self.executables.to_vec()
    }
    /// check if the given path has the file or dir or symlink or executable
    pub fn has(&self, x: &String) -> bool {
        let files_founded = self.files.contains(x);
        let dirs_founded = self.dirs.contains(x);
        let executable_founded = self.executables.contains(x);
        let symlink_founded = self.symlinks.contains(x);
        files_founded || dirs_founded || executable_founded || symlink_founded
    }
    /// create a new file if it does not exist, return true if created, false if already exists
    pub fn touch(&self, p: &Path) -> bool {
        if p.exists() {
            false
        } else {
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(p)
                .is_ok()
        }
    }
    /// create a new file with content if it does not exist, return true if created, false if already exists
    pub fn touch_with_content(&self, p: &Path, content: String) -> bool {
        if p.exists() {
            false
        } else {
            let file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(p)
                .expect("failed to create_new");
            let mut writer = BufWriter::new(file);
            writer
                .write_all(content.as_bytes())
                .expect("failed to write");
            writer.flush().expect("failed to Write");
            true
        }
    }
    /// create a new file if it does not exist, return true if created, false if already exists
    pub fn create(&self, p: &Path) -> bool {
        OpenOptions::new().write(true).open(p).is_ok()
    }
    /// append content to the file, return true if successful, false if file does not exist
    pub fn add(&self, p: &Path, content: String) -> bool {
        let file = OpenOptions::new()
            .append(true)
            .open(p)
            .expect("failed to open file");
        let mut writer = BufWriter::new(file);
        if self.file_content(p).is_empty() {
            writeln!(writer, "{}", content).expect("");
        } else {
            writeln!(writer, "\n{}", content).expect("");
        }
        writer.flush().expect("failed to Write");
        true
    }
    /// overwrite content to the file, return true if successful, false if file does not exist
    pub fn erase(&self, p: &Path, new_content: String) -> bool {
        if p.exists() && new_content.is_empty() {
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(p)
                .is_ok()
        } else if p.is_file() {
            let file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(p)
                .expect("");
            let mut writer = BufWriter::new(file);
            writer
                .write_all(new_content.as_bytes())
                .expect("failed to write");
            writer.flush().expect("failed to Write");
            true
        } else {
            false
        }
    }
    /// remove a directory and all its contents, return true if successful, false if directory does not exist
    pub fn remove_dir(&self, p: &Path) -> bool {
        if p.is_dir() {
            std::fs::remove_dir_all(p).is_ok()
        } else {
            false
        }
    }
    /// remove a file, return true if successful, false if file does not exist
    pub fn remove_file(&self, p: &Path) -> bool {
        if p.is_file() {
            std::fs::remove_file(p).is_ok()
        } else {
            false
        }
    }
    /// remove a symlink, return true if successful, false if symlink does not exist
    pub fn remove_symlink(&self, p: &Path) -> bool {
        if p.is_symlink() {
            std::fs::remove_file(p).is_ok()
        } else {
            false   
        }
    }
    /// remove an executable, return true if successful, false if executable does not exist
    pub fn remove_executable(&self, p: &Path) -> bool {
        if p.is_executable() {      
            std::fs::remove_file(p).is_ok()
        } else {
            false
        }
    }
    /// get the content of the file, return empty string if file does not exist
    pub fn file_content(&self, p: &Path) -> String {
        if p.exists() {
            read_to_string(p).expect("failed to read file content")
        } else {
            String::new()
        }
    }
    /// get the content of the file, return empty string if file does not exist
    pub fn push_content(&self, p: &Path) -> String {
        if p.exists() {
            read_to_string(p).expect("failed to read file content")
        } else {
            String::new()
        }
    }
    /// get the content of the file by index, return empty string if file does not exist
    pub fn get_file_content(&self, index: usize) -> String {
        if let Some(x) = self.files.get(index) {
            read_to_string(x).expect("failed to read file content")
        } else {
            String::new()
        }
    }
    /// load the content of the files by indexes, return self
    pub fn load(&mut self, indexes: Vec<usize>) -> &mut Self {
        for i in indexes {
            if let Some(x) = self.files.get(i) {
                self.loaded.insert(
                    x.to_string(),
                    read_to_string(x).expect("failed to read file content"),
                );
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fs() -> QwxFileSystem {
        QwxFileSystem::new(&Path::new("."))
    }
    #[test]
    pub fn test() {
        let x = fs();
        assert!(x.files.is_empty().eq(&false));
        assert!(x.dirs.is_empty().eq(&false));
        assert!(x.has(&"README.md".to_string()));
        assert!(x.has(&"src".to_string()));
    }

    #[test]
    pub fn test_create_and_remove() {
        let x = fs();
        let p = Path::new("a");
        if p.exists() {
            assert!(x.remove_file(p));
        }
        assert!(x.touch(&p));
        assert!(x.file_content(&p).is_empty());
        assert!(x.add(&p, "hello".to_string()));
        assert!(x.file_content(&p).eq(&"hello\n".to_string()));
        assert!(x.erase(&p, String::new()));
        assert!(x.file_content(&p).eq(&String::new()));
        assert!(fs().has(&String::from("a")));
        assert!(x.remove_file(&p));
        assert!(fs().has(&String::from("a")).eq(&false));
        assert!(fs().touch_with_content(&p, "hello".to_string()).eq(&true));
        assert!(fs().has(&String::from("a")).eq(&true));
        assert!(fs().file_content(&p).eq(&"hello".to_string()));
        assert!(fs().add(&p, "hello".to_string()));
        assert!(fs().file_content(&p).eq(&"hello\nhello\n".to_string()));
        assert!(fs().remove_file(&p));
        assert!(fs().has(&String::from("a")).eq(&false));
    }
    #[test]
    pub fn test_load() {
        let mut f = fs();
        assert!(f.load(vec![0]).loaded.is_empty().eq(&false));
        assert!(f.load(vec![0]).loaded.len().eq(&1));
        assert!(f.load(vec![0, 1, 2, 3]).loaded.len().eq(&4));
    }
}
