// Windows release, remove the console window
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io;
use std::io::Write;
use tauri::Emitter;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn steam_is_installed() -> Result<String, String> {
    let mut key = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
    let mut subkey = key
        .open_subkey("SOFTWARE\\Wow6432Node\\Valve\\Steam")
        .unwrap();
    let mut value: String = subkey.get_value("InstallPath").unwrap();
    if value.is_empty() {
        //try 32 bit
        key = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
        subkey = key.open_subkey("SOFTWARE\\Valve\\Steam").unwrap();
        value = subkey.get_value("InstallPath").unwrap();
    }
    if value.is_empty() {
        return Err("Steam nebyl nalezen".into());
    }
    Ok(value.into())
}

#[tauri::command]
async fn get_steam_vdf() -> Result<String, String> {
    let mut key = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
    let mut subkey = key
        .open_subkey("SOFTWARE\\Wow6432Node\\Valve\\Steam")
        .unwrap();
    let mut value: String = subkey.get_value("InstallPath").unwrap();
    if value.is_empty() {
        //try 32 bit
        key = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
        subkey = key.open_subkey("SOFTWARE\\Valve\\Steam").unwrap();
        value = subkey.get_value("InstallPath").unwrap();
    }
    if value.is_empty() {
        return Err("Steam nebyl nalezen".into());
    }
    // get steam library - steam path + steamapps/libraryfolders.vdf
    let mut steam_library = value.clone();
    steam_library.push_str("\\steamapps\\libraryfolders.vdf");

    // read libraryfolders.vdf
    let file = std::fs::read_to_string(steam_library);
    if file.is_err() {
        return Err("Nepodařilo se otevřít soubor libraryfolders.vdf".into());
    }
    let file = file.unwrap();
    Ok(file)
}

#[tauri::command]
async fn file_exists(path: String) -> Result<String, String> {
    let path = std::path::Path::new(&path);
    if path.exists() {
        Ok("true".into())
    } else {
        Ok("false".into())
    }
}

#[tauri::command]
async fn download(url: String, filename: String, window: tauri::Window) -> Result<String, String> {
    // get temp path
    let path = std::env::temp_dir().join(filename.clone());
    println!("Saving downloaded file to: {:?}", path);

    // Use reqwest's async client instead of blocking
    let client = reqwest::Client::new();

    // get the response
    let response = match client.get(&url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            return Err(format!("Nepodařilo se stáhnout soubor: {}", e));
        }
    };

    // check resp status code
    if !response.status().is_success() {
        let status = response.status();
        let error_msg = format!(
            "Server vrátil chybu {} při stahování: {}",
            status.as_u16(),
            url
        );
        return Err(error_msg);
    }

    // Check content type
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap_or(""))
        .unwrap_or("");

    // Expected content types for EXE and ZIP files
    let valid_exe_types = [
        "application/vnd.microsoft.portable-executable",
        "application/x-msdownload",
        "application/octet-stream",
    ];
    let valid_zip_types = [
        "application/zip",
        "application/x-zip-compressed",
        "application/octet-stream",
    ];

    let extension = std::path::Path::new(&filename)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("")
        .to_lowercase();

    // content type == expected file extension
    let is_valid = match extension.as_str() {
        "exe" => valid_exe_types.contains(&content_type),
        "zip" => valid_zip_types.contains(&content_type),
        _ => content_type == "application/octet-stream" || content_type.starts_with("application/"),
    };

    // check if the content type is valid
    if !is_valid {
        return Err(format!(
            "Neplatný typ souboru: {}. Očekávaný typ pro {}: {}",
            content_type,
            if extension == "exe" {
                "EXE"
            } else if extension == "zip" {
                "ZIP"
            } else {
                "binární soubor"
            },
            if extension == "exe" {
                "application/x-msdownload"
            } else {
                "application/zip"
            }
        ));
    }

    // Get content length for progress calculation
    let total_size = response.content_length().unwrap_or(0);

    // Create the file
    let mut file = match std::fs::File::create(&path) {
        Ok(f) => f,
        Err(e) => {
            return Err(format!("Nepodařilo se vytvořit soubor: {}", e));
        }
    };

    // Use proper byte stream handling
    use futures_util::StreamExt;
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    let start_time = std::time::Instant::now();
    let mut last_update = std::time::Instant::now();

    // Process the stream in chunks
    while let Some(chunk_result) = stream.next().await {
        let chunk = match chunk_result {
            Ok(chunk) => chunk,
            Err(e) => return Err(format!("Chyba při stahování: {}", e)),
        };

        // Write the chunk to the file
        if let Err(e) = file.write_all(&chunk) {
            return Err(format!("Nepodařilo se zapsat do souboru: {}", e));
        }

        // Update progress
        downloaded += chunk.len() as u64;

        // Send progress update every 100ms
        if last_update.elapsed().as_millis() > 100 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 {
                downloaded as f64 / elapsed / 1024.0 / 1024.0
            } else {
                0.0
            };

            let percentage = if total_size > 0 {
                (downloaded as f64 / total_size as f64) * 100.0
            } else {
                0.0
            };

            // Emit progress event
            let _ = window.emit(
                "download:progress",
                Some(serde_json::json!({
                    "downloaded": downloaded,
                    "total": total_size,
                    "percentage": percentage,
                    "speed": speed
                })),
            );

            last_update = std::time::Instant::now();
        }
    }

    // Emit final progress
    let elapsed = start_time.elapsed().as_secs_f64();
    let speed = if elapsed > 0.0 {
        downloaded as f64 / elapsed / 1024.0 / 1024.0
    } else {
        0.0
    };
    let _ = window.emit(
        "download:progress",
        Some(serde_json::json!({
            "downloaded": downloaded,
            "total": total_size,
            "percentage": 100.0,
            "speed": speed
        })),
    );

    Ok("true".into())
}

#[tauri::command]
async fn unzip_file(path: String) -> Result<Vec<String>, String> {
    //make the path %temp% + path
    let path = std::env::temp_dir().join(path);
    let fname = std::path::Path::new(&path);
    if !fname.exists() {
        return Err("Soubor neexistuje".into());
    }
    println!("Unzipping file: {:?}", fname);

    let file = match std::fs::File::open(fname) {
        Ok(f) => f,
        Err(e) => return Err(format!("Nepodařilo se otevřít zip soubor: {}", e)),
    };

    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(e) => return Err(format!("Nepodařilo se rozbalit zip soubor: {}", e)),
    };

    let mut extracted_files = vec![];

    let zip_name = std::path::Path::new(fname)
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("");
    // temp + hash
    let output_dir = std::env::temp_dir().join(zip_name);

    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(f) => f,
            Err(e) => return Err(format!("Nepodařilo se přečíst soubor z archivu: {}", e)),
        };

        let filepath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        let outpath = output_dir.join(&filepath);

        if (*file.name()).ends_with('/') {
            //folder
            println!(
                "Folder {} extracted to \"{}\"",
                file.name(),
                outpath.display()
            );
            if let Err(e) = std::fs::create_dir_all(&outpath) {
                return Err(format!("Nepodařilo se vytvořit složku: {}", e));
            }
        } else {
            //file
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    if let Err(e) = std::fs::create_dir_all(&p) {
                        return Err(format!("Nepodařilo se vytvořit nadřazenou složku: {}", e));
                    }
                }
            }

            let mut outfile = match std::fs::File::create(&outpath) {
                Ok(f) => f,
                Err(e) => return Err(format!("Nepodařilo se vytvořit soubor: {}", e)),
            };

            if let Err(e) = io::copy(&mut file, &mut outfile) {
                return Err(format!("Nepodařilo se rozbalit soubor: {}", e));
            }

            if let Some(outpath_str) = outpath.to_str() {
                extracted_files.push(outpath_str.replace("/", "\\"));
            }
        }
        println!("Extracted {:?} to {:?}", file.name(), outpath);
    }

    // remove the zip file
    if let Err(e) = std::fs::remove_file(fname) {
        return Err(format!("Nepodařilo se odstranit zip soubor: {}", e));
    }

    //check if is removed
    if fname.exists() {
        return Err("Nepodařilo se odstranit zip".into());
    }

    // return array of extracted files
    Ok(extracted_files)
}

#[tauri::command]
async fn patch_file(mut path: String, patch: String) -> Result<String, String> {
    let xdelta3 = include_bytes!("./xdelta.exe");
    use futures::channel::oneshot;
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    use std::thread;
    let no_win: u32 = 0x08000000;

    // spawn xdelta in temp
    let mut temp_path = std::env::temp_dir();
    temp_path.push("xdelta3.exe");
    // if xdelta3.exe doesn't exist, create it
    if !temp_path.exists() {
        let mut file = std::fs::File::create(&temp_path).unwrap();
        file.write_all(xdelta3).unwrap();
    }
    //if the path ends with .backup, rename it to be without the .backup
    if path.ends_with(".backup") {
        let old_path = std::path::Path::new(&path);
        let new_path = old_path.with_extension("");
        std::fs::rename(old_path, new_path.clone()).unwrap();
        //change the path to the new path
        path = new_path.to_str().unwrap().to_string();
    }
    // let output be path + ".patched"
    let output = format!("{}.patched", path);

    // Clone values for the thread
    let temp_path_clone = temp_path.clone();
    let path_clone = path.clone();
    let output_clone = output.clone();
    let patch_clone = patch.clone();

    // Create a oneshot channel for communicating the result
    let (sender, receiver) = oneshot::channel();

    // Spawn a thread to execute the command
    thread::spawn(move || {
        let mut command = Command::new(temp_path_clone);
        command.arg("-d");
        command.arg("-f");
        command.arg("-s");
        command.arg(path_clone);
        command.arg(patch_clone);
        command.arg(output_clone);
        command.creation_flags(no_win);

        println!("Running command: {:?}", command);
        let out = command.output();
        let _ = sender.send(out);
    });

    // Await the result from the thread
    let out = receiver
        .await
        .map_err(|_| "Thread communication failed".to_string())?;

    match out {
        Ok(out) => {
            let output_xdelta = String::from_utf8(out.stderr).unwrap();
            if output_xdelta.is_empty() {
                // backup the original file
                let backup = format!("{}.backup", path);
                std::fs::copy(&path, &backup).unwrap();
                // rename the patched file to the original file
                let path = std::path::Path::new(&path);
                // set patch path, its path + .patched
                let output_path = std::path::Path::new(&output);
                println!("try to rename {:?} to {:?}", output_path, path);
                match std::fs::rename(output_path, &path) {
                    Ok(_) => {
                        if !path.exists() {
                            println!("Rename wasn't successful");
                            return Err("Nepodařilo se přejmenovat soubor".into());
                        }
                        Ok("ok".into())
                    }
                    Err(e) => {
                        println!("An error occurred: {}", e);
                        Err("Failed to rename file".into())
                    }
                }
            } else {
                Err(output_xdelta)
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn get_md5(path: String) -> Result<String, String> {
    let content = std::fs::read(path);
    match content {
        Ok(content) => {
            let hash = md5::compute(content);
            //return the hash as string
            Ok(format!("{:x}", hash))
        }
        Err(_) => Err("error".into()),
    }
}
#[tauri::command]
async fn backup_renew(path: String) -> Result<String, String> {
    //if the path ends with .backup, rename it to be without the .backup
    if path.ends_with(".backup") {
        let old_path = std::path::Path::new(&path);
        let new_path = old_path.with_extension("");

        match std::fs::rename(old_path, new_path) {
            Ok(_) => return Ok("Ok".into()),
            Err(e) => return Err(e.to_string()),
        }
    }
    return Err("doesnt exists".into());
}
#[tauri::command]
async fn create_sha256_hash_from_timestamp_with_salt(timestamp: &str) -> Result<String, String> {
    use dotenv_codegen::dotenv;
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(timestamp);
    //get salt from .env file
    let salt = dotenv!("SALT");
    hasher.update(salt);
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

#[tauri::command]
async fn delete_temps(folder: String) -> Result<String, String> {
    // also del the xdelta3.exe
    let mut temp_path = std::env::temp_dir();
    temp_path.push("xdelta3.exe");
    if temp_path.exists() {
        std::fs::remove_file(temp_path.clone()).unwrap();
        // check if it was removed
        if temp_path.exists() {
            return Err("Nepodařilo se odstranit xdelta3.exe".into());
        }
    }
    // delete the folder (temp folder + folder name)
    let mut folder_del = std::env::temp_dir();
    folder_del.push(folder);
    if folder_del.exists() {
        match std::fs::remove_dir_all(&folder_del) {
            Ok(_) => {}
            Err(e) => return Err(format!("Nepodařilo se odstranit složku: {}", e)),
        }
    }
    Ok("ok".into())
}

#[tauri::command]
async fn copy_and_replace(from: String, to: String) -> Result<String, String> {
    let from = std::path::Path::new(&from);
    let to = std::path::Path::new(&to);
    if !from.exists() || !from.is_file() {
        return Err("File doesn't exist".into());
    }

    // Create parent directories if they don't exist
    if let Some(parent) = to.parent() {
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return Err(format!(
                    "Nepodařilo se vytvořit adresářovou strukturu: {}",
                    e
                ));
            }
        }
    }

    if to.exists() {
        // rename to backup
        let backup = format!("{}.backup", to.to_str().unwrap());
        match std::fs::rename(to, backup) {
            Ok(_) => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    match std::fs::copy(from, to) {
        Ok(_) => Ok("Ok".into()),
        Err(e) => Err(e.to_string()),
    }
}
#[tauri::command]
async fn get_temp_dir() -> Result<String, String> {
    Ok(std::env::temp_dir().to_str().unwrap().to_string())
}
#[tauri::command]
async fn delete_file(path: String) -> Result<String, String> {
    let path = std::path::Path::new(&path);
    if !path.exists() {
        return Err("soubor neexistuje".into());
    }
    match std::fs::remove_file(path) {
        Ok(_) => Ok("ok".into()),
        Err(e) => Err(e.to_string()),
    }
}

#[derive(serde::Serialize)]
struct FileCheckResult {
    file: String,
    reason: String,
}

#[tauri::command]
async fn check_files(files: Vec<String>) -> Result<Vec<FileCheckResult>, String> {
    use std::fs::OpenOptions;
    use std::os::windows::fs::OpenOptionsExt;

    let mut results = Vec::new();

    for file_path in files {
        let path = std::path::Path::new(&file_path);

        if !path.exists() {
            continue;
        }

        let attempt = OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(0x0)
            .open(&path);

        match attempt {
            Ok(_file) => {}
            Err(error) => {
                if let Some(code) = error.raw_os_error() {
                    let reason = match code {
                        32 | 33 => "Already in use",
                        5 => "No rw",
                        _ => "Other",
                    };
                    if reason != "Other" { // only report targeted reasons
                        let file_name = path
                            .file_name()
                            .unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
                            .to_string_lossy()
                            .to_string();
                        results.push(FileCheckResult {
                            file: file_name,
                            reason: reason.to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(results)
}
#[tauri::command]
async fn update_the_app(url: String, window: tauri::Window) -> Result<String, String> {
    let current_exe_name = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .file_name()
        .ok_or("Failed to get file name")?
        .to_string_lossy()
        .into_owned();
    //download the update
    match download(url, current_exe_name.clone(), window).await {
        Ok(_) => {
            println!("Download ok");
        }
        Err(e) => {
            eprintln!("Download error: {}", e);
            return Err(e);
        }
    }
    //get temp dir with the downloaded file
    let temp_dir = std::env::temp_dir().join(current_exe_name.clone());
    // spawn cmd that will replace the current exe with the downloaded
    let mut command = std::process::Command::new("cmd");
    command.arg("/C");
    // exit the old app
    command.arg("taskkill");
    command.arg("/f");
    command.arg("/im");
    command.arg(current_exe_name.clone());
    command.arg("&&");
    // del the old exe
    command.arg("del");
    command.arg("/f");
    command.arg("/q");
    command.arg(current_exe_name.clone());
    command.arg("&&");
    // move from tmp to current dir
    command.arg("move");
    command.arg("/y"); // overwrite
    command.arg(temp_dir);
    command.arg(current_exe_name.clone());
    command.arg("&&");
    // start the new exe
    command.arg("start");
    command.arg(current_exe_name);
    command.spawn().map_err(|e| e.to_string())?;

    println!("Running command: {:?}", command);
    Ok("ok".into())
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            steam_is_installed,
            get_steam_vdf,
            file_exists,
            download,
            unzip_file,
            patch_file,
            get_md5,
            backup_renew,
            create_sha256_hash_from_timestamp_with_salt,
            delete_temps,
            copy_and_replace,
            get_temp_dir,
            delete_file,
            check_files,
            update_the_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
