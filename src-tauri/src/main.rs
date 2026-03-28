// Windows release, remove the console window
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use futures::channel::oneshot;
use sha2::Digest;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime};
use tauri::Emitter;
use tokio::time::sleep;

#[cfg(not(windows))]
use std::env;
#[cfg(not(windows))]
use std::path::PathBuf;

#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;
#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[tauri::command]
async fn steam_is_installed() -> Result<String, String> {
    #[cfg(not(windows))]
    {
        // use which steam
        let check = std::process::Command::new("which").arg("steam").output();
        match check {
            Ok(output) => {
                if output.status.success() {
                    Ok("true".into())
                } else {
                    Err("Steam not installed".into())
                }
            }
            Err(_) => Err("Steam not installed".into()),
        }
    }
    #[cfg(windows)]
    {
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
            return Err("Steam not found".into());
        }
        Ok(value.into())
    }
}

#[tauri::command]
async fn get_steam_vdf() -> Result<String, String> {
    #[cfg(not(windows))]
    {
        let home = env::var("HOME").map_err(|_| "Could not find HOME directory".to_string())?;

        // Standard
        let mut vdf_path = PathBuf::from(&home);
        vdf_path.push(".local/share/Steam/steamapps/libraryfolders.vdf");

        // Flatpak
        if !vdf_path.exists() {
            vdf_path = PathBuf::from(&home);
            vdf_path.push(
                ".var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/libraryfolders.vdf",
            );
        }

        if !vdf_path.exists() {
            return Err("Steam vdf file not found".into());
        }

        std::fs::read_to_string(vdf_path).map_err(|e| format!("Failed reading VDF: {}", e))
    }
    #[cfg(windows)]
    {
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
            return Err("Steam not found".into());
        }
        // get steam library - steam path + steamapps/libraryfolders.vdf
        let mut steam_library = value.clone();
        steam_library.push_str("\\steamapps\\libraryfolders.vdf");

        // read libraryfolders.vdf
        let file = std::fs::read_to_string(steam_library);
        if file.is_err() {
            return Err("Failed to read libraryfolders.vdf".into());
        }
        let file = file.unwrap();
        Ok(file)
    }
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

    static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    println!("Using User-Agent: {}", APP_USER_AGENT);
    // Use reqwest's async client instead of blocking
    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .map_err(|e| format!("Nepodařilo se vytvořit HTTP klienta: {}", e))?;

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
                #[cfg(windows)]
                extracted_files.push(outpath_str.replace("/", "\\"));
                #[cfg(not(windows))]
                extracted_files.push(outpath_str.to_string());
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
    const NO_WINDOW_FLAG: u32 = 0x08000000;
    let temp_path = {
        #[cfg(windows)]
        {
            let mut temp_path = std::env::temp_dir();
            let xdelta3 = include_bytes!("./xdelta.exe");

            // spawn xdelta in temp
            temp_path.push("xdelta3.exe");
            // if xdelta3.exe doesn't exist, create it
            if !temp_path.exists() {
                std::fs::write(&temp_path, xdelta3)
                    .map_err(|e| format!("Nepodařilo se vytvořit xdelta3.exe: {}", e))?;
            }
            temp_path
        }
        #[cfg(not(windows))]
        {
            // check if xdelta3 is installed
            let check = std::process::Command::new("which").arg("xdelta3").output();
            match check {
                Ok(output) => {
                    if !output.status.success() {
                        return Err("Package xdelta3 není nainstalován. Prosím nainstalujte jej a zkuste to znovu.".into());
                    }
                    std::path::PathBuf::from(
                        String::from_utf8_lossy(&output.stdout).trim().to_string(),
                    )
                }
                Err(_) => {
                    return Err(
                        "Package xdelta3 není nainstalován. Prosím nainstalujte jej a zkuste to znovu."
                            .into(),
                    );
                }
            }
        }
    };

    //if the path ends with .backup, rename it to be without the .backup
    if path.ends_with(".backup") {
        let old_path = std::path::Path::new(&path);
        let new_path = old_path.with_extension("");
        tokio::fs::rename(old_path, &new_path)
            .await
            .map_err(|e| format!("Chyba při přejmenování zálohy: {}", e))?;
        path = new_path.to_str().unwrap_or(&path).to_string();
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

    // wait for xdelta (av checks etc.)
    #[cfg(windows)]
    {
        let mut retries = 0;
        const MAX_RETRIES: u32 = 10;
        loop {
            let mut command = Command::new(&temp_path);
            command.arg("--help");
            #[cfg(windows)]
            {
                command.creation_flags(NO_WINDOW_FLAG);
            }
            match command.output() {
                Ok(_) => break,
                Err(e) => {
                    if retries >= MAX_RETRIES {
                        return Err(format!("Nepodařilo se spustit xdelta3. {}", e));
                    }
                    if !temp_path.exists() {
                        return Err("Soubor xdelta3 byl smazán.".into());
                    }
                    sleep(Duration::from_millis(500)).await;
                    retries += 1;
                }
            }
        }
    }

    // Spawn a thread to execute the command
    thread::spawn(move || {
        let mut command = Command::new(temp_path_clone);
        command.arg("-d");
        command.arg("-f");
        command.arg("-s");
        command.arg(path_clone);
        command.arg(patch_clone);
        command.arg(output_clone);
        #[cfg(windows)]
        {
            command.creation_flags(NO_WINDOW_FLAG);
        }

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
async fn get_last_modified(path: String) -> Result<String, String> {
    let metadata = std::fs::metadata(path);
    match metadata {
        Ok(metadata) => {
            let modified_time = metadata.modified();
            match modified_time {
                Ok(time) => {
                    let duration = time
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default();
                    Ok(duration.as_secs().to_string())
                }
                Err(_) => Err("error".into()),
            }
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
    let mut hasher = sha2::Sha256::new();
    hasher.update(timestamp);
    let salt = env!("SALT");
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
async fn check_files(
    files: Vec<String>,
    base_path: String,
    is_zip: bool,
) -> Result<Vec<FileCheckResult>, String> {
    let base_path = std::path::Path::new(&base_path);
    if is_zip {
        #[cfg(windows)]
        {
            const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x02000000; // to open dirs
                                                                // for zip patches, check if dir is writable
            let attempt = OpenOptions::new()
                .write(true)
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
                .open(&base_path);
            match attempt {
                Ok(_dir) => {
                    return Ok(vec![]);
                }
                Err(error) => {
                    if let Some(code) = error.raw_os_error() {
                        let reason = match code {
                            32 | 33 => "Already in use",
                            5 => "No rw",
                            _ => "Other",
                        };
                        if reason != "Other" {
                            return Ok(vec![FileCheckResult {
                                file: base_path
                                    .file_name()
                                    .unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
                                    .to_string_lossy()
                                    .to_string(),
                                reason: reason.to_string(),
                            }]);
                        }
                    }
                }
            }
        }
        #[cfg(not(windows))]
        {
            // for zip patches, check if dir is writable
            if OpenOptions::new().write(true).open(&base_path).is_err() {
                return Ok(vec![FileCheckResult {
                    file: base_path
                        .file_name()
                        .unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
                        .to_string_lossy()
                        .to_string(),
                    reason: "No rw".into(),
                }]);
            } else {
                return Ok(vec![]);
            }
        }
    }

    let mut results = Vec::new();

    for file_path in files {
        let path = std::path::Path::new(&file_path);

        if !path.exists() {
            continue;
        }
        #[cfg(windows)]
        {
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
                        if reason != "Other" {
                            // only report targeted reasons
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
        #[cfg(not(windows))]
        {
            if OpenOptions::new()
                .read(true)
                .write(true)
                .open(&path)
                .is_err()
            {
                let file_name = path
                    .file_name()
                    .unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
                    .to_string_lossy()
                    .to_string();
                results.push(FileCheckResult {
                    file: file_name,
                    reason: "No rw".into(),
                });
            }
        }
    }

    Ok(results)
}
#[tauri::command]
async fn update_the_app(url: String, window: tauri::Window) -> Result<String, String> {
    #[cfg(not(windows))]
    {
        return Err(
            "Aktualizace je podporována pouze na Windows. Nejnovější verzi si stáhněte z GitHubu."
                .into(),
        );
    }

    let current_exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
    let current_exe_name = current_exe_path
        .file_name()
        .ok_or("Failed to get file name")?
        .to_string_lossy()
        .into_owned();

    match download(url, current_exe_name.clone(), window.clone()).await {
        Ok(_) => println!("Download ok"),
        Err(e) => {
            eprintln!("Download error: {}", e);
            return Err(e);
        }
    }

    let temp_dir = std::env::temp_dir();
    let temp_file_path = temp_dir.join(&current_exe_name);
    let bat_file_path = temp_dir.join("updater.bat");

    let bat_content = format!(
        "@echo off\n\
        timeout /t 2 /nobreak > NUL\n\
        move /y \"{}\" \"{}\"\n\
        start \"\" \"{}\"\n\
        del \"%~f0\"",
        temp_file_path.display(),
        current_exe_path.display(),
        current_exe_path.display()
    );

    std::fs::write(&bat_file_path, bat_content)
        .map_err(|e| format!("Failed to write updater.bat: {}", e))?;

    // spawn .bat
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        std::process::Command::new("cmd")
            .arg("/C")
            .arg(&bat_file_path)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("Failed to spawn updater: {}", e))?;
    }
    std::process::exit(0);
}

#[tauri::command]
fn check_xdelta() -> bool {
    #[cfg(windows)]
    {
        return true; // is bundled
    }
    #[cfg(not(windows))]
    {
        let check = std::process::Command::new("which").arg("xdelta3").output();
        match check {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            steam_is_installed,
            get_steam_vdf,
            file_exists,
            download,
            unzip_file,
            patch_file,
            get_md5,
            get_last_modified,
            backup_renew,
            create_sha256_hash_from_timestamp_with_salt,
            delete_temps,
            copy_and_replace,
            get_temp_dir,
            delete_file,
            check_files,
            update_the_app,
            check_xdelta
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
