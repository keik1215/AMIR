use chrono::Utc;
use directories::BaseDirs;
use futures_util::StreamExt;
use rand::{
    distr::{Alphanumeric, SampleString},
    rng,
};
use reqwest::Response;
use serde::Serialize;
use serde_json::{from_reader, json, to_writer, Value};
use std::{
    cmp::min,
    fs::{self, File},
    io::{copy, BufReader, BufWriter, Read, SeekFrom, Write},
    path::Path,
    sync::atomic::AtomicBool,
};
use std::{
    io::Seek,
    sync::atomic::Ordering::{Relaxed, Release},
};
use tauri::{AppHandle, Emitter};
use zip::ZipArchive;

static DEAD: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Serialize)]
struct MessagePayload {
    message: String,
}

#[derive(Clone, Serialize)]
struct FileMessagePayload {
    name: String,
    size: String,
    id: String,
}

#[derive(Clone, Serialize)]
struct FileChangePayload {
    name: String,
    progress: u32,
    id: String,
}
#[derive(Clone, Serialize)]
struct ZipExtractStartPayload {
    name: String,
    files: u32,
    id: String,
}
#[derive(Clone, Serialize)]
struct ZipExtractFilePayload {
    name: String,
    progress: u32,
    count: u32,
    now: u32,
    id: String,
}

#[derive(Clone, Serialize)]
struct FileErrPayload {
    message: String,
    id: String,
}

#[derive(Clone, Serialize)]
struct ProfErrPayload {
    message: String,
}

#[tauri::command]
fn app_data() -> String {
    let mut result: String = "".to_string();
    if let Some(dir) = BaseDirs::new() {
        result = dir.config_dir().to_str().unwrap().to_string()
    }
    result
}

#[tauri::command]
fn diagnosis_java() -> bool {
    match std::process::Command::new("java").output() {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[tauri::command]
async fn get_forge(forge: String) {
    let versions: Vec<&str> = forge.split("-").collect();
    let mc_ver = versions[0];
    let fg_ver = versions[2];
    let forge_jar: Response = reqwest::get(
        format!("https://maven.minecraftforge.net/net/minecraftforge/forge/{0}-{1}/forge-{0}-{1}-installer.jar", mc_ver, fg_ver)
    ).await.expect("err");
    let bytes = forge_jar.bytes().await.expect("err");
    let mut file = fs::File::options()
        .write(true)
        .read(true)
        .create(true)
        .open(&format!("forge-{0}-{1}-installer.jar", mc_ver, fg_ver))
        .unwrap();
    copy(&mut bytes.as_ref(), &mut file).expect("err");
    std::process::Command::new("java")
        .arg("-jar")
        .arg(format!("forge-{0}-{1}-installer.jar", mc_ver, fg_ver))
        .output()
        .expect("err");
}

#[tauri::command]
fn diagnosis_forge(version: String, mc_path: String) -> bool {
    let mut result = false;
    if mc_path.eq("") {
        result = true
    }
    if !result {
        let dot_minecraft = Path::new(&mc_path);
        if dot_minecraft.is_dir() {
            let need_version = dot_minecraft.join("versions").join(version);
            if need_version.exists() {
                result = true;
            }
        }
    }
    result
}

#[tauri::command]
async fn dl_direct_zip(app_handle: AppHandle, url: String, dir: String, is_other: bool) {
    let name = rand_str();
    let extract_name = rand_str();
    emit_change(
        &app_handle,
        format!("Downloading zip file from {}", url).to_string(),
    );
    let directory_zip: Response;
    match reqwest::get(url).await {
        Ok(r) => {
            directory_zip = r;
        }
        Err(e) => {
            emit_errfile(&app_handle, e.to_string(), name.clone());
            emit_err(&app_handle, e.to_string());
            return;
        }
    }
    let total_size = directory_zip.content_length().expect("err");
    app_handle
        .emit(
            "newfile",
            FileMessagePayload {
                name: "<Zip file>".to_string(),
                size: fmt_size(total_size),
                id: name.clone(),
            },
        )
        .expect("err");
    emit_change(
        &app_handle,
        format!("Making temporary zip file: _{}.zip", name).to_string(),
    );
    let mut file = fs::File::options()
        .write(true)
        .read(true)
        .create(true)
        .open(&format!("_{}.zip", name))
        .expect("err");
    let mut byte_stream = directory_zip.bytes_stream();
    let mut got: u64 = 0;
    while let Some(item) = byte_stream.next().await {
        if DEAD.load(Relaxed) {
            fs::remove_file(&format!("_{}.zip", name)).expect("err");
            DEAD.store(false, Release);
            return;
        }
        let chunk = item.expect("err");
        got = min(got + chunk.len() as u64, total_size);
        match file.write_all(&chunk) {
            Ok(_) => {
                app_handle
                    .emit(
                        "filechange",
                        FileChangePayload {
                            name: "<Zip file>".to_string(),
                            id: name.clone(),
                            progress: (got * 100 / total_size) as u32,
                        },
                    )
                    .expect("err");
            }
            Err(e) => {
                emit_errfile(&app_handle, e.to_string(), name.clone());
                return;
            }
        };
    }
    let mut zip: ZipArchive<File>;
    match zip::ZipArchive::new(file) {
        Ok(r) => {
            zip = r;
            app_handle
                .emit(
                    "zipstart",
                    ZipExtractStartPayload {
                        name: "<Zip extraction>".to_string(),
                        files: zip.len() as u32,
                        id: extract_name.clone(),
                    },
                )
                .expect("err");
        }
        Err(e) => {
            emit_errfile(&app_handle, e.to_string(), name.clone());
            return;
        }
    }
    emit_change(
        &app_handle,
        format!("Starting zip extraction ({} files)", zip.len()).to_string(),
    );
    let dir_path = Path::new(&dir).join("mods");
    let dir_path_other = Path::new(&dir);
    if is_other {
        if !dir_path_other.exists() {
            fs::create_dir(dir_path_other).expect("err");
        }
    } else {
        if !dir_path.exists() {
            fs::create_dir(dir_path.clone()).expect("err");
        }
    }
    let len = zip.len();
    for i in 0..len {
        if DEAD.load(Relaxed) {
            DEAD.store(false, Release);
            fs::remove_file(&format!("_{}.zip", name)).expect("err");
            return;
        }
        let mut file = zip.by_index(i).expect("err");
        let mut directory: Vec<&str> = file.name().split("/").collect();
        if !file.is_file() {
            app_handle
                .emit(
                    "zipfile",
                    ZipExtractFilePayload {
                        name: "directory: ".to_string()
                            + file
                                .mangled_name()
                                .as_path()
                                .file_name()
                                .unwrap()
                                .to_str()
                                .unwrap(),
                        count: len as u32,
                        progress: ((100 * (i + 1)) as u32) / len as u32,
                        now: (i + 1) as u32,
                        id: extract_name.clone(),
                    },
                )
                .expect("err");
            emit_change(
                &app_handle,
                format!("Directory skipped {} ({}/{})", file.name(), i + 1, len),
            );
            continue;
        }
        let without_top: Vec<&str> = directory.clone();
        directory.pop().expect("err");
        if is_other {
            fs::create_dir_all(dir_path_other.join(directory.join("/"))).expect("err");
        } else {
            fs::create_dir_all(dir_path.join(directory.join("/"))).expect("err");
        }
        emit_change(
            &app_handle,
            format!(
                "Starting extraction {} ({}/{})",
                file.mangled_name()
                    .as_path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap(),
                i + 1,
                len
            ),
        );
        let mut new_downloaded: File;
        if is_other {
            match fs::File::options()
                .write(true)
                .read(false)
                .create(true)
                .open(dir_path_other.join(without_top.join("/")))
            {
                Ok(r) => {
                    new_downloaded = r;
                }
                Err(e) => {
                    emit_errfile(&app_handle, e.to_string(), extract_name.clone());
                    return;
                }
            }
        } else {
            match fs::File::options()
                .write(true)
                .read(false)
                .create(true)
                .open(dir_path.join(without_top.join("/")))
            {
                Ok(r) => {
                    new_downloaded = r;
                }
                Err(e) => {
                    emit_errfile(&app_handle, e.to_string(), extract_name.clone());
                    return;
                }
            }
        }
        copy(file.by_ref(), &mut new_downloaded).expect("err");
        app_handle
            .emit(
                "zipfile",
                ZipExtractFilePayload {
                    name: file
                        .mangled_name()
                        .as_path()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                    count: len as u32,
                    id: extract_name.clone(),
                    progress: ((100 * (i + 1)) as u32) / len as u32,
                    now: (i + 1) as u32,
                },
            )
            .expect("err");
    }
    fs::remove_file(format!("_{}.zip", name)).expect("err");
}

#[tauri::command]
fn dead_end() {
    DEAD.store(true, Release);
}

#[tauri::command]
async fn dl_direct_jar(app_handle: AppHandle, url: String, dir: String, name: String) {
    let id = rand_str();
    emit_change(
        &app_handle,
        format!("Downloading jar file from {}", url).to_string(),
    );
    let jar: Response;
    match reqwest::get(url).await {
        Ok(r) => {
            jar = r;
        }
        Err(e) => {
            emit_errfile(&app_handle, e.to_string(), id.clone());
            return;
        }
    }
    let total_size = jar.content_length().expect("err");
    let mut byte_stream = jar.bytes_stream();
    let mut got: u64 = 0;
    let dir_path = Path::new(&dir).join("mods");
    if !dir_path.exists() {
        fs::create_dir(dir_path.clone()).expect("err");
    }
    let mut file = fs::File::options()
        .write(true)
        .read(true)
        .create(true)
        .open(dir_path.join(name.clone()))
        .expect("err");
    app_handle
        .emit(
            "newfile",
            FileMessagePayload {
                name: name.clone(),
                size: fmt_size(total_size),
                id: id.clone(),
            },
        )
        .expect("err");
    while let Some(item) = byte_stream.next().await {
        if DEAD.load(Relaxed) {
            DEAD.store(false, Release);
            fs::remove_file(dir_path.join(name.clone())).expect("err");
            return;
        }
        let chunk = item.expect("err");
        got = min(got + chunk.len() as u64, total_size);
        match file.write_all(&chunk) {
            Ok(_) => {
                app_handle
                    .emit(
                        "filechange",
                        FileChangePayload {
                            name: name.clone(),
                            id: id.clone(),
                            progress: (100 * got / total_size) as u32,
                        },
                    )
                    .expect("err");
            }
            Err(e) => {
                emit_errfile(&app_handle, e.to_string(), id);
                return;
            }
        };
    }
}

#[tauri::command]
async fn make_profile(
    app_handle: AppHandle,
    name: String,
    mc_dir: String,
    game_dir: String,
    version: String,
) {
    let profiles_path = Path::new(&mc_dir).join("launcher_profiles.json");
    let profiles;
    match fs::File::open(profiles_path.clone()) {
        Ok(r) => {
            profiles = BufReader::new(r);
        }
        Err(e) => {
            app_handle
                .emit(
                    "proferr",
                    ProfErrPayload {
                        message: format!(
                            "Failed in getting launcher_profiles.json: {}",
                            e.to_string()
                        )
                        .to_string(),
                    },
                )
                .expect("err");
            return;
        }
    }
    let json = from_reader::<BufReader<fs::File>, Value>(profiles);
    let mut json: Value = match json {
        Ok(o) => o,
        Err(e) => {
            app_handle
                .emit(
                    "proferr",
                    ProfErrPayload {
                        message: format!(
                            "Failed in parsing launcher_profiles.json: {}",
                            e.to_string()
                        )
                        .to_string(),
                    },
                )
                .expect("err");
            return;
        }
    };
    json["profiles"][name] = json!({
        "created": Utc::now().to_string(),
        "icon" : "TNT",
        "lastUsed": "1970-01-01T00:00:00.000Z",
        "lastVersionId": version,
        "name": name,
        "type": "custom",
        "gameDir": game_dir
    });
    let mut writer = BufWriter::new(
        fs::File::options()
            .read(false)
            .write(true)
            .append(false)
            .open(profiles_path)
            .unwrap(),
    );
    writer.seek(SeekFrom::Start(0)).expect("err");
    to_writer(writer, &json).expect("err");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            app_data,
            diagnosis_java,
            diagnosis_forge,
            get_forge,
            dl_direct_zip,
            dl_direct_jar,
            make_profile,
            dead_end
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn fmt_size(size: u64) -> String {
    let size_text = size.to_string();
    if size_text.len() >= 4 {
        if size_text.len() >= 7 {
            if size_text.len() >= 10 {
                return (size / 1000000000).to_string() + "GB";
            }
            return (size / 1000000).to_string() + "MB";
        }
        return (size / 1000).to_string() + "KB";
    }
    size_text + "B"
}

fn rand_str() -> String {
    Alphanumeric.sample_string(&mut rng(), 10)
}

fn emit_err(app_handle: &AppHandle, e: String) {
    app_handle
        .emit("error", MessagePayload { message: e })
        .expect("err");
}

fn emit_errfile(app_handle: &AppHandle, e: String, id: String) {
    app_handle
        .emit("errfile", FileErrPayload { message: e, id: id })
        .expect("err");
}

fn emit_change(app_handle: &AppHandle, e: String) {
    app_handle
        .emit("change", MessagePayload { message: e })
        .expect("err");
}
