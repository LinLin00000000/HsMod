use dialoguer::{Input, Select};
use include_dir::{include_dir, Dir, File};
use log::debug;
use std::{env, fs, io::Result, path::Path};
use walkdir::WalkDir;

static RESOURCE_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/resources");

fn main() -> Result<()> {
    if env::args().any(|arg| arg == "--debug") || cfg!(debug_assertions) {
        env::set_var("RUST_LOG", "debug");
    }
    env_logger::init();

    let hearthstone_directory =
        find_hearthstone_directory().unwrap_or_else(|| prompt_for_directory());
    println!("炉石传说目录: {}", hearthstone_directory);

    let action = prompt_for_action();
    perform_action(Path::new(&hearthstone_directory), action)?;

    let _ = std::io::stdin().read_line(&mut String::new());
    Ok(())
}

fn install(output_dir: &Path) -> Result<()> {
    fn extract_resource(file: &File, output_dir: &Path) -> Result<()> {
        fn ensure_directory_exists(path: &Path) -> Result<()> {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
            } else {
                Ok(())
            }
        }

        let output_path = output_dir.join(file.path());
        ensure_directory_exists(&output_path)?;
        fs::write(output_path, file.contents())
    }

    for entry in RESOURCE_DIR.find("**/*").unwrap() {
        debug!("Found {}", entry.path().display());

        if let Some(file) = entry.as_file() {
            extract_resource(file, &output_dir)?;
        }
    }

    Ok(())
}

fn uninstall(output_dir: &Path) -> Result<()> {
    // 逆序遍历资源目录，确保子目录的文件先被删除
    let resources: Vec<_> = RESOURCE_DIR.find("**/*").unwrap().collect();

    for entry in resources.into_iter().rev() {
        let full_path = output_dir.join(entry.path());

        if let Some(_file) = entry.as_file() {
            // 如果是文件且存在，则删除
            if full_path.exists() {
                debug!("Deleting file: {}", full_path.display());
                fs::remove_file(full_path)?;
            }
        } else if let Some(_dir) = entry.as_dir() {
            // 如果是目录且为空，则删除
            if full_path.exists() && full_path.read_dir()?.next().is_none() {
                debug!("Deleting directory: {}", full_path.display());
                fs::remove_dir(full_path)?;
            }
        }
    }

    Ok(())
}

fn find_hearthstone_directory() -> Option<String> {
    fn find_file_directory(target: &str, paths: Vec<&str>, max_depth: usize) -> Option<String> {
        for path in paths {
            let result = WalkDir::new(path)
                .max_depth(max_depth)
                .into_iter()
                .filter_map(|e| e.ok()) // 处理可能的遍历错误
                .find(|e| {
                    e.file_name().to_string_lossy().eq_ignore_ascii_case(target)
                        && e.file_type().is_file()
                }); // 查找文件名匹配且为文件的项

            if let Some(entry) = result {
                return entry
                    .path()
                    .parent()
                    .map(|p| p.to_string_lossy().to_string());
            }
        }
        None
    }

    let target_file = "Hearthstone.exe";
    let max_depth = 3; // 设置搜索的最大深度

    // 直接使用字符串字面量定义搜索路径数组
    let search_paths = vec![
        "C:\\Program Files (x86)",
        "C:\\Program Files",
        "D:\\",
        "E:\\",
    ];

    find_file_directory(target_file, search_paths, max_depth)
}

fn prompt_for_directory() -> String {
    loop {
        let input: String = Input::new()
            .with_prompt("未找到 Hearthstone.exe。请输入炉石传说的目录路径")
            .interact_text()
            .unwrap();

        let path = Path::new(&input);
        let is_file = path
            .file_name()
            .map_or(false, |name| name == "Hearthstone.exe");

        let directory = if is_file {
            path.parent().unwrap().to_path_buf()
        } else {
            path.to_path_buf()
        };

        if directory.join("Hearthstone.exe").exists() {
            return directory.to_string_lossy().into_owned();
        } else {
            println!("无效的目录，请重新输入。");
        }
    }
}

fn prompt_for_action() -> String {
    let actions = vec!["安装", "卸载"];
    let selection = Select::new()
        .with_prompt("请选择要执行的操作")
        .items(&actions)
        .default(0)
        .interact_opt()
        .unwrap();

    actions[selection.unwrap_or(0)].to_string()
}

fn perform_action(directory: &Path, action: String) -> Result<()> {
    match action.as_str() {
        "安装" => match install(directory) {
            Ok(_) => println!("安装成功。"),
            Err(e) => println!("安装失败: {}", e),
        },
        "卸载" => match uninstall(directory) {
            Ok(_) => println!("卸载成功。"),
            Err(e) => println!("卸载失败: {}", e),
        },
        _ => unreachable!(),
    }
    Ok(())
}
