pub mod archive {
    use std::{fs, io::BufReader};

    const SUPPORTED_ARCHIVES: [&str; 1] = [".zip"];

    /// Checks whether the given path is a supported archive. Supported as in supported by
    /// file-explorer-tui
    /// See SUPPORTED_ARCHIVES const for all supported archives
    pub fn is_path_supported_archive(path: &String) -> bool {
        for supported_archive in SUPPORTED_ARCHIVES {
            if path.ends_with(supported_archive) {
                return true;
            }
        }
        return false;
    }

    pub fn get_files_of_archive(path: &String) -> Vec<String> {
        let file = fs::File::open(path).unwrap();
        let reader = BufReader::new(file);

        let mut archive = zip::ZipArchive::new(reader).unwrap();

        for i in 0..archive.len() {
            let file = archive.by_index(i).unwrap();

            // TODO: Warning about enclosed name relevant if we are only reading?
            println!("{:?}", file.name());
        }

        return vec!["test".to_string()];
    }
}
