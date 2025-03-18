pub mod file {
    use std::{cmp::Ordering, fs, path::Path};

    use ratatui::text::Text;

    #[derive(Clone)]
    pub struct File {
        pub display_name: String,
        pub full_path: String,
        pub is_dir: bool,
    }

    impl Ord for File {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            let self_starts_with_dot = self.display_name.starts_with(".");
            let other_starts_with_dot = other.display_name.starts_with(".");

            if self_starts_with_dot && other_starts_with_dot {
                return self
                    .display_name
                    .to_lowercase()
                    .cmp(&other.display_name.to_lowercase());
            }

            if self_starts_with_dot {
                return Ordering::Greater;
            }

            if other_starts_with_dot {
                return Ordering::Less;
            }

            self.display_name
                .to_lowercase()
                .cmp(&other.display_name.to_lowercase())
        }
    }

    impl PartialOrd for File {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl PartialEq for File {
        fn eq(&self, other: &Self) -> bool {
            self.display_name.to_lowercase() == other.display_name.to_lowercase()
        }
    }

    impl Eq for File {}

    impl ToString for File {
        fn to_string(&self) -> String {
            self.display_name.to_string()
        }
    }

    impl From<File> for String {
        fn from(value: File) -> String {
            value.display_name
        }
    }

    impl From<File> for Text<'_> {
        fn from(value: File) -> Self {
            Text::raw(value.display_name)
        }
    }

    pub fn get_files_for_dir(dir: &String) -> Vec<File> {
        let read_dir_result = fs::read_dir(dir).expect("Can read from dir");

        let files: Vec<File> = read_dir_result
            .into_iter()
            .map(|file| {
                let dir_entry = file.expect("can unwrap file");
                let full_path = dir_entry.path().to_string_lossy().to_string();
                let splitted: Vec<&str> = full_path.split("/").collect();
                let (last, _) = splitted
                    .split_last()
                    .expect("Should be able to split to get relative path");

                let is_dir = is_path_directory(&full_path);

                let display_name = if is_dir {
                    last.to_string() + "/"
                } else {
                    last.to_string()
                };

                return File {
                    display_name,
                    full_path,
                    is_dir,
                };
            })
            .collect();
        return files;
    }

    // TODO: Write unit tests for this function
    pub fn get_parent_dir(current_path: &String) -> String {
        let splitted_path: Vec<&str> = current_path.split("/").collect();
        let split_last_result = splitted_path.split_last();
        return match split_last_result {
            None => current_path.to_string(),
            Some(result) => {
                let (_, elements) = result;
                if elements.len() == 1 && elements[0] == "" {
                    return String::from("/");
                }
                return elements.join("/");
            }
        };
    }

    pub fn is_path_directory(path: &String) -> bool {
        Path::new(path).is_dir()
    }

    pub fn sort_file_paths_dirs_first_then_files(files: &Vec<File>) -> Vec<File> {
        let dirs: Vec<File> = files
            .iter()
            .filter(|file| file.is_dir)
            .map(|file| file.clone())
            .collect();
        let files: Vec<&File> = files.iter().filter(|file| !file.is_dir).collect();

        let mut everything_together: Vec<File> =
            dirs.iter().chain(files).map(|file| file.clone()).collect();
        everything_together.sort();
        return everything_together;
    }

    /// Deletes the given file. If its just a file, it will be deleted. If its a directory, the
    /// entire directory will be deleted recursively.
    pub fn delete_file(file: &File) -> Result<(), std::io::Error> {
        let file_path = &file.full_path;
        let is_path_directory = is_path_directory(file_path);
        if is_path_directory {
            fs::remove_dir_all(file_path)
        } else {
            fs::remove_file(file_path)
        }
    }
}
