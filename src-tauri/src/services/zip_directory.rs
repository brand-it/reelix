// use log::debug;
// use std::fs::File;
// use std::io::{Read, Write};
// use std::path::Path;
// use walkdir::WalkDir;
// use zip::{result::ZipError, write::SimpleFileOptions, CompressionMethod, ZipWriter};

// pub fn zip_dir(src_dir: &Path, dst_file: &Path, method: CompressionMethod) -> Result<(), ZipError> {
//     if !Path::new(src_dir).is_dir() {
//         return Err(ZipError::FileNotFound);
//     }

//     let path = Path::new(&dst_file);
//     let file = File::create(path).unwrap();

//     let walkdir = WalkDir::new(src_dir);
//     let dir_entries = walkdir.into_iter().filter_map(|e| e.ok());

//     let mut zip = ZipWriter::new(file);
//     let options = SimpleFileOptions::default()
//         .compression_method(method)
//         .unix_permissions(0o755);

//     let prefix = Path::new(src_dir);
//     let mut buffer = Vec::new();
//     for entry in dir_entries {
//         let path = entry.path();
//         let name = path.strip_prefix(prefix).unwrap();
//         let path_as_string = name.to_str().map(str::to_owned).unwrap();

//         // Write file or directory explicitly
//         // Some unzip tools unzip files with directory paths correctly, some do not!
//         if path.is_file() {
//             debug!("adding file {path:?} as {name:?} ...");
//             zip.start_file(path_as_string, options)?;
//             let mut f = File::open(path)?;

//             f.read_to_end(&mut buffer)?;
//             zip.write_all(&buffer)?;
//             buffer.clear();
//         } else if !name.as_os_str().is_empty() {
//             // Only if not root! Avoids path spec / warning
//             // and map name conversion failed error on unzip
//             debug!("adding dir {path_as_string:?} as {name:?} ...");
//             zip.add_directory(path_as_string, options)?;
//         }
//     }
//     zip.finish()?;
//     Ok(())
// }
