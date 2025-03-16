pub mod file {
    use std::{fs, path::Path};

    use ratatui::text::Text;

    #[derive(Clone)]
    pub struct File {
        pub display_name: String,
        pub full_path: String,
    }

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
                // i have a feeling this is not the way to go
                let dir_entry = file.expect("can unwrap file");
                let full_path = dir_entry.path().to_string_lossy().to_string();
                let splitted: Vec<&str> = full_path.split("/").collect();
                let (last, _) = splitted
                    .split_last()
                    .expect("Should be able to split to get relative path");

                let display_name = if Path::new(&full_path).is_dir() {
                    last.to_string() + "/"
                } else {
                    last.to_string()
                };

                return File {
                    display_name,
                    full_path,
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
}
