// Windows release, remove the console window
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io;
use std::io::Write;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn steam_is_installed() -> Result<String, String> {
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
fn get_steam_vdf() -> Result<String, String> {
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
fn file_exists(path: String) -> Result<String, String> {
    let path = std::path::Path::new(&path);
    if path.exists() {
        Ok("true".into())
    } else {
        Ok("false".into())
    }
}

#[tauri::command]
fn download(url: String, filename: String) -> Result<String, String> {
    use std::io::Write;
    let path = std::env::temp_dir();
    let path = path.join(filename);
    let response = reqwest::blocking::get(&url);
    if response.is_err() {
        return Err("Nepodařilo se stáhnout soubor".into());
    }
    let response = response.unwrap();
    if !response.status().is_success() {
        return Err("Nepodařilo se stáhnout soubor".into());
    }
    let file = std::fs::File::create(path);
    if file.is_err() {
        return Err("Nepodařilo se vytvořit soubor".into());
    }
    let mut file = file.unwrap();
    let content = response.bytes();
    if content.is_err() {
        return Err("Nepodařilo se získat obsah souboru".into());
    }
    let content = content.unwrap();
    if let Err(e) = file.write_all(&content) {
        eprintln!("Error writing to file: {}", e);
        return Err("Nepodařilo se zapsat do souboru".into());
    }
    Ok("true".into())
}

#[tauri::command]
fn unzip_file(path: String) -> Result<Vec<String>, String> {
    //make the path %temp% + path
    let path = std::env::temp_dir().join(path);
    let fname = std::path::Path::new(&path);
    if !fname.exists() {
        return Err("Soubor neexistuje".into());
    }
    println!("Unzipping file: {:?}", fname);
    let file = std::fs::File::open(fname).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    //array of extracted files
    let mut extracted_files = vec![];
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let filepath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };
        // Get the name of the zip file without the .zip extension
        let zip_name = std::path::Path::new(fname)
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("");
        // Use a subdirectory of the temporary directory as the output directory
        let output_dir = std::env::temp_dir().join(zip_name);
        let outpath = output_dir.join(filepath);

        if (*file.name()).ends_with('/') {
            //folder
            println!(
                "File {} extracted to \"{}\"",
                file.name(),
                outpath.display()
            );
            std::fs::create_dir_all(&outpath).unwrap();
        } else {
            //file
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(&p).unwrap();
                }
            }
            let mut outfile = std::fs::File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
            let outpath_str = outpath.to_str().unwrap().replace("/", "\\");
            extracted_files.push(outpath_str);
        }
        println!("Extracted {:?} {:?}", file.name(), outpath);
    }
    // remove the zip file
    std::fs::remove_file(fname).unwrap();
    //check if is removed
    if fname.exists() {
        return Err("Nepodařilo se odstranit zip".into());
    }
    // return array of extracted files
    Ok(extracted_files)
}

#[tauri::command]
fn patch_file(mut path: String, patch: String) -> Result<String, String> {
    let xdelta3 = include_bytes!("./xdelta.exe");
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    let no_win: u32 = 0x08000000;
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

    let mut command = Command::new(temp_path);
    command.arg("-d");
    command.arg("-f");
    command.arg("-s");
    command.arg(path.clone());
    command.arg(patch);
    command.arg(output.clone());
    command.creation_flags(no_win);
    println!("Running command: {:?}", command);
    let out = command.output();
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
fn get_md5(path: String) -> Result<String, String> {
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
fn backup_renew(path: String) -> Result<String, String> {
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
fn create_sha256_hash_from_timestamp_with_salt(timestamp: &str) -> Result<String, String> {
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
fn delete_temps(delete: Vec<&str>, folder: String) -> Result<String, String> {
    let paths = delete.to_vec();
    for path in paths {
        let path = std::path::Path::new(path);
        if path.exists() {
            std::fs::remove_file(path).unwrap();
            // check if it was removed
            if path.exists() {
                let err = format!("Nepodařilo se odstranit soubor {}", path.display());
                return Err(err.into());
            }
        }
    }
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
        std::fs::remove_dir_all(folder_del.clone()).unwrap();
        // check if it was removed
        if folder_del.exists() {
            return Err("Nepodařilo se odstranit složku".into());
        }
    }
    Ok("ok".into())
}

#[tauri::command]
fn copy_and_replace(from: String, to: String) -> Result<String, String> {
    let from = std::path::Path::new(&from);
    let to = std::path::Path::new(&to);
    if !from.exists() || !from.is_file() {
        return Err("File doesn't exist".into());
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
fn get_temp_dir() -> Result<String, String> {
    Ok(std::env::temp_dir().to_str().unwrap().to_string())
}
#[tauri::command]
fn delete_file(path: String) -> Result<String, String> {
    let path = std::path::Path::new(&path);
    if !path.exists() {
        return Err("soubor neexistuje".into());
    }
    match std::fs::remove_file(path) {
        Ok(_) => Ok("ok".into()),
        Err(e) => Err(e.to_string()),
    }
}
#[tauri::command]
fn update_the_app(url: String) -> Result<String, String> {
    let current_exe_name = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .file_name()
        .ok_or("Failed to get file name")?
        .to_string_lossy()
        .into_owned();
    //download the update
    match download(url, current_exe_name.clone()) {
        Ok(_) => {}
        Err(e) => return Err(e),
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
            update_the_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
