use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process;
use walkdir::WalkDir;

// ==========================================
// Настройка консоли Windows для UTF-8
// ==========================================
#[cfg(windows)]
fn setup_windows_console() {
    use windows::Win32::System::Console::{
        GetConsoleMode, GetStdHandle, SetConsoleCP, SetConsoleMode, SetConsoleOutputCP,
        CONSOLE_MODE, ENABLE_VIRTUAL_TERMINAL_PROCESSING, STD_OUTPUT_HANDLE,
    };
    use windows::Win32::Globalization::CP_UTF8;

    unsafe {
        let out_result = SetConsoleOutputCP(CP_UTF8);
        let in_result = SetConsoleCP(CP_UTF8);

        if out_result.is_err() || in_result.is_err() {
            eprintln!("[Warning] Failed to set UTF-8 encoding for console.");
            eprintln!("          Russian/Chinese characters may not display correctly.");
            eprintln!("          Please use Windows Terminal for best experience.");
        }

        if let Ok(handle) = GetStdHandle(STD_OUTPUT_HANDLE) {
            let mut mode = CONSOLE_MODE(0);
            if GetConsoleMode(handle, &mut mode).is_ok() {
                let new_mode = CONSOLE_MODE(mode.0 | ENABLE_VIRTUAL_TERMINAL_PROCESSING.0);
                let _ = SetConsoleMode(handle, new_mode);
            }
        }
    }
}

#[cfg(not(windows))]
fn setup_windows_console() {}

// ==========================================
// Языки и Локализация
// ==========================================
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lang {
    RU,
    EN,
    ZH,
}

impl std::str::FromStr for Lang {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "RU" | "РУ" => Ok(Lang::RU),
            "EN" | "АНГЛ" => Ok(Lang::EN),
            "ZH" | "КИТ" => Ok(Lang::ZH),
            _ => Err(format!("Invalid language: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Msg {
    Footer, MenuTitle,
    M1, M2, M3, M4, M5, M6, M7, M8, M9, M10, M11, M12, M13, M14,
    PromptChoice, PromptSource, PromptOutput,
    CurrentExts, PromptAddExt, PromptRemExt, UseAllFiles,
    CurrentSep, PromptSep,
    CurrentExcl, PromptAddExcl, PromptRemExcl, PromptClearExcl,
    CurrentIncl, PromptAddIncl, PromptRemIncl, PromptClearIncl,
    PreviewTitle, PreviewNext, RunCopyPrompt, Yes, InvalidChoice,
    Utf8Warn, ResetSuccess, SaveSuccess, ExitMsg,
    StatusOk, StatusExcl, StatusSys, StatusExt, StatusNotIncl,
    UsageDetails,
}

pub fn tr(msg: Msg, lang: Lang) -> String {
    match (msg, lang) {
        (Msg::Footer, Lang::RU) => "Сделано в России. Программа создана полностью на языке программирования Rust. Автор программы: М. Мих. Мир".into(),
        (Msg::Footer, Lang::EN) => "Made in Russia. The program is created entirely in the Rust programming language. Author: M. Mikh. Mir".into(),
        (Msg::Footer, Lang::ZH) => "在俄罗斯制造。该程序完全使用 Rust 编程语言创建。作者：M. Mikh. Mir".into(),

        (Msg::MenuTitle, Lang::RU) => "ГЛАВНОЕ МЕНЮ".into(),
        (Msg::MenuTitle, Lang::EN) => "MAIN MENU".into(),
        (Msg::MenuTitle, Lang::ZH) => "主菜单".into(),

        (Msg::M1, Lang::RU) => "Выбрать исходную папку".into(),
        (Msg::M1, Lang::EN) => "Select source folder".into(),
        (Msg::M1, Lang::ZH) => "选择源文件夹".into(),

        (Msg::M2, Lang::RU) => "Выбрать определённые папки".into(),
        (Msg::M2, Lang::EN) => "Select specific folders".into(),
        (Msg::M2, Lang::ZH) => "选择特定文件夹".into(),

        (Msg::M3, Lang::RU) => "Настроить расширения".into(),
        (Msg::M3, Lang::EN) => "Configure extensions".into(),
        (Msg::M3, Lang::ZH) => "配置扩展名".into(),

        (Msg::M4, Lang::RU) => "Настроить разделитель (0-5)".into(),
        (Msg::M4, Lang::EN) => "Configure separator (0-5)".into(),
        (Msg::M4, Lang::ZH) => "配置分隔符 (0-5)".into(),

        (Msg::M5, Lang::RU) => "Игнорировать служебные папки".into(),
        (Msg::M5, Lang::EN) => "Ignore system directories".into(),
        (Msg::M5, Lang::ZH) => "忽略系统目录".into(),

        (Msg::M6, Lang::RU) => "Настроить исключения по путям".into(),
        (Msg::M6, Lang::EN) => "Configure path exclusions".into(),
        (Msg::M6, Lang::ZH) => "配置路径排除".into(),

        (Msg::M7, Lang::RU) => "Выбрать выходной файл".into(),
        (Msg::M7, Lang::EN) => "Select output file".into(),
        (Msg::M7, Lang::ZH) => "选择输出文件".into(),

        (Msg::M8, Lang::RU) => "Запустить копирование".into(),
        (Msg::M8, Lang::EN) => "Run copy".into(),
        (Msg::M8, Lang::ZH) => "运行复制".into(),

        (Msg::M9, Lang::RU) => "Показать превью списка файлов".into(),
        (Msg::M9, Lang::EN) => "Show file list preview".into(),
        (Msg::M9, Lang::ZH) => "显示文件列表预览".into(),

        (Msg::M10, Lang::RU) => "Сбросить настройки".into(),
        (Msg::M10, Lang::EN) => "Reset settings".into(),
        (Msg::M10, Lang::ZH) => "重置设置".into(),

        (Msg::M11, Lang::RU) => "Сохранить как умолчания".into(),
        (Msg::M11, Lang::EN) => "Save as defaults".into(),
        (Msg::M11, Lang::ZH) => "保存为默认".into(),

        (Msg::M12, Lang::RU) => "Показать инструкцию".into(),
        (Msg::M12, Lang::EN) => "Show instructions".into(),
        (Msg::M12, Lang::ZH) => "显示说明".into(),

        (Msg::M13, Lang::RU) => "Язык интерфейса [Рус] [Eng] [中文]".into(),
        (Msg::M13, Lang::EN) => "Interface language [Рус] [Eng] [中文]".into(),
        (Msg::M13, Lang::ZH) => "界面语言 [Рус] [Eng] [中文]".into(),

        (Msg::M14, Lang::RU) => "Выход".into(),
        (Msg::M14, Lang::EN) => "Exit".into(),
        (Msg::M14, Lang::ZH) => "退出".into(),

        (Msg::PromptChoice, Lang::RU) => "Ваш выбор: ".into(),
        (Msg::PromptChoice, Lang::EN) => "Your choice: ".into(),
        (Msg::PromptChoice, Lang::ZH) => "您的选择: ".into(),
        (Msg::PromptSource, Lang::RU) => "Введите путь к исходной папке: ".into(),
        (Msg::PromptSource, Lang::EN) => "Enter source folder path: ".into(),
        (Msg::PromptSource, Lang::ZH) => "输入源文件夹路径: ".into(),
        (Msg::PromptOutput, Lang::RU) => "Введите имя выходного файла: ".into(),
        (Msg::PromptOutput, Lang::EN) => "Enter output file name: ".into(),
        (Msg::PromptOutput, Lang::ZH) => "输入输出文件名: ".into(),

        (Msg::CurrentExts, Lang::RU) => "1. Добавить\n2. Удалить\n3. Переключить 'Все файлы'".into(),
        (Msg::CurrentExts, Lang::EN) => "1. Add\n2. Remove\n3. Toggle 'All files'".into(),
        (Msg::CurrentExts, Lang::ZH) => "1. 添加\n2. 删除\n3. 切换'所有文件'".into(),
        (Msg::PromptAddExt, Lang::RU) => "Введите расширение (напр. .md): ".into(),
        (Msg::PromptAddExt, Lang::EN) => "Enter extension (e.g. .md): ".into(),
        (Msg::PromptAddExt, Lang::ZH) => "输入扩展名 (例如 .md): ".into(),
        (Msg::PromptRemExt, Lang::RU) => "Введите расширение для удаления: ".into(),
        (Msg::PromptRemExt, Lang::EN) => "Enter extension to remove: ".into(),
        (Msg::PromptRemExt, Lang::ZH) => "输入要删除的扩展名: ".into(),
        (Msg::UseAllFiles, Lang::RU) => "Режим 'Все файлы' переключен.".into(),
        (Msg::UseAllFiles, Lang::EN) => "Toggled 'All files' mode.".into(),
        (Msg::UseAllFiles, Lang::ZH) => "已切换'所有文件'模式".into(),

        (Msg::CurrentSep, Lang::RU) => "Текущий разделитель: ".into(),
        (Msg::CurrentSep, Lang::EN) => "Current separator: ".into(),
        (Msg::CurrentSep, Lang::ZH) => "当前分隔符: ".into(),
        (Msg::PromptSep, Lang::RU) => "Введите число пустых строк (0-5): ".into(),
        (Msg::PromptSep, Lang::EN) => "Enter number of empty lines (0-5): ".into(),
        (Msg::PromptSep, Lang::ZH) => "输入空行数 (0-5): ".into(),

        (Msg::CurrentExcl, Lang::RU) => "1. Добавить\n2. Удалить\n3. Очистить все".into(),
        (Msg::CurrentExcl, Lang::EN) => "1. Add\n2. Remove\n3. Clear all".into(),
        (Msg::CurrentExcl, Lang::ZH) => "1. 添加\n2. 删除\n3. 清除所有".into(),
        (Msg::PromptAddExcl, Lang::RU) => "Введите путь для исключения: ".into(),
        (Msg::PromptAddExcl, Lang::EN) => "Enter path to exclude: ".into(),
        (Msg::PromptAddExcl, Lang::ZH) => "输入要排除的路径: ".into(),
        (Msg::PromptRemExcl, Lang::RU) => "Введите путь для удаления: ".into(),
        (Msg::PromptRemExcl, Lang::EN) => "Enter path to remove: ".into(),
        (Msg::PromptRemExcl, Lang::ZH) => "输入要删除的路径: ".into(),
        (Msg::PromptClearExcl, Lang::RU) => "Список исключений очищен.".into(),
        (Msg::PromptClearExcl, Lang::EN) => "Exclusion list cleared.".into(),
        (Msg::PromptClearExcl, Lang::ZH) => "排除列表已清除".into(),

        (Msg::CurrentIncl, Lang::RU) => "1. Добавить\n2. Удалить\n3. Очистить все".into(),
        (Msg::CurrentIncl, Lang::EN) => "1. Add\n2. Remove\n3. Clear all".into(),
        (Msg::CurrentIncl, Lang::ZH) => "1. 添加\n2. 删除\n3. 清除所有".into(),
        (Msg::PromptAddIncl, Lang::RU) => "Введите путь к папке для включения: ".into(),
        (Msg::PromptAddIncl, Lang::EN) => "Enter folder path to include: ".into(),
        (Msg::PromptAddIncl, Lang::ZH) => "输入要包含的文件夹路径: ".into(),
        (Msg::PromptRemIncl, Lang::RU) => "Введите путь для удаления: ".into(),
        (Msg::PromptRemIncl, Lang::EN) => "Enter path to remove: ".into(),
        (Msg::PromptRemIncl, Lang::ZH) => "输入要删除的路径: ".into(),
        (Msg::PromptClearIncl, Lang::RU) => "Список включённых папок очищен.".into(),
        (Msg::PromptClearIncl, Lang::EN) => "Included folders list cleared.".into(),
        (Msg::PromptClearIncl, Lang::ZH) => "包含文件夹列表已清除".into(),

        (Msg::PreviewTitle, Lang::RU) => "ПРЕВЬЮ: будет скопировано {} файлов".into(),
        (Msg::PreviewTitle, Lang::EN) => "PREVIEW: {} files will be copied".into(),
        (Msg::PreviewTitle, Lang::ZH) => "预览: 将复制 {} 个文件".into(),
        (Msg::PreviewNext, Lang::RU) => "Показать следующие 50 [Enter], или Q для выхода".into(),
        (Msg::PreviewNext, Lang::EN) => "Show next 50 [Enter], or Q to quit".into(),
        (Msg::PreviewNext, Lang::ZH) => "显示下50个 [回车], 或 Q 退出".into(),

        (Msg::RunCopyPrompt, Lang::RU) => "Выполнить копирование этих файлов? (Y/N): ".into(),
        (Msg::RunCopyPrompt, Lang::EN) => "Run copy for these files? (Y/N): ".into(),
        (Msg::RunCopyPrompt, Lang::ZH) => "复制这些文件吗？(Y/N): ".into(),
        (Msg::Yes, Lang::RU) => "Y".into(), (Msg::Yes, Lang::EN) => "Y".into(), (Msg::Yes, Lang::ZH) => "Y".into(),

        (Msg::InvalidChoice, Lang::RU) => "Неверный выбор.".into(),
        (Msg::InvalidChoice, Lang::EN) => "Invalid choice.".into(),
        (Msg::InvalidChoice, Lang::ZH) => "无效选择".into(),
        (Msg::Utf8Warn, Lang::RU) => "Пропуск (не UTF-8)".into(),
        (Msg::Utf8Warn, Lang::EN) => "Skipping (not UTF-8)".into(),
        (Msg::Utf8Warn, Lang::ZH) => "跳过 (非 UTF-8)".into(),

        (Msg::ResetSuccess, Lang::RU) => "Настройки сброшены.".into(),
        (Msg::ResetSuccess, Lang::EN) => "Settings reset.".into(),
        (Msg::ResetSuccess, Lang::ZH) => "设置已重置".into(),
        (Msg::SaveSuccess, Lang::RU) => "Настройки сохранены.".into(),
        (Msg::SaveSuccess, Lang::EN) => "Settings saved.".into(),
        (Msg::SaveSuccess, Lang::ZH) => "设置已保存".into(),
        (Msg::ExitMsg, Lang::RU) => "До свидания!".into(),
        (Msg::ExitMsg, Lang::EN) => "Goodbye!".into(),
        (Msg::ExitMsg, Lang::ZH) => "再见!".into(),

        (Msg::StatusOk, Lang::RU) => "[OK]".into(), (Msg::StatusOk, Lang::EN) => "[OK]".into(), (Msg::StatusOk, Lang::ZH) => "[OK]".into(),
        (Msg::StatusExcl, Lang::RU) => "[ИСКЛ]".into(), (Msg::StatusExcl, Lang::EN) => "[EXCL]".into(), (Msg::StatusExcl, Lang::ZH) => "[排除]".into(),
        (Msg::StatusSys, Lang::RU) => "[СИСТ]".into(), (Msg::StatusSys, Lang::EN) => "[SYS]".into(), (Msg::StatusSys, Lang::ZH) => "[系统]".into(),
        (Msg::StatusExt, Lang::RU) => "[РАСШ]".into(), (Msg::StatusExt, Lang::EN) => "[EXT]".into(), (Msg::StatusExt, Lang::ZH) => "[扩展]".into(),
        (Msg::StatusNotIncl, Lang::RU) => "[НЕ ВКЛ]".into(), (Msg::StatusNotIncl, Lang::EN) => "[NOT INCL]".into(), (Msg::StatusNotIncl, Lang::ZH) => "[未包含]".into(),

        (Msg::UsageDetails, Lang::RU) => "ИНСТРУКЦИЯ:\nПрограмма собирает код из монорепозитория в один текстовый файл.\nПоддерживаются превью (--preview), исключения (--exclude), выбор конкретных папок (--only-dirs) и настройки конфигурации.".into(),
        (Msg::UsageDetails, Lang::EN) => "INSTRUCTIONS:\nCollects code from monorepo into a single text file.\nSupports preview (--preview), exclusions (--exclude), specific folders (--only-dirs) and config settings.".into(),
        (Msg::UsageDetails, Lang::ZH) => "说明:\n从 monorepo 收集代码到单个文本文件。\n支持预览 (--preview)、排除 (--exclude)、特定文件夹 (--only-dirs) 和配置设置.".into(),
    }
}

// ==========================================
// Конфигурация
// ==========================================
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Config {
    default_extensions: Vec<String>,
    enabled_extensions: Vec<String>,
    use_all_files: bool,
    separator_lines: u8,
    ignore_system_dirs: bool,
    exclude_paths: Vec<String>,
    include_dirs: Vec<String>,
    language: String,
    last_source: String,
    last_output: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_extensions: vec![".rs".into(), ".toml".into()],
            enabled_extensions: vec![".rs".into(), ".toml".into()],
            use_all_files: false,
            separator_lines: 2,
            ignore_system_dirs: true,
            exclude_paths: vec![],
            include_dirs: vec![],
            language: "RU".into(),
            last_source: ".".into(),
            last_output: "merged_output.txt".into(),
        }
    }
}

fn get_config_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".copy_prog");
    let _ = fs::create_dir_all(&path);
    path.push("config.json");
    path
}

fn load_config() -> Config {
    let path = get_config_path();
    if path.exists() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str(&data) {
                return cfg;
            }
        }
    }
    Config::default()
}

fn save_config(cfg: &Config) {
    let path = get_config_path();
    if let Ok(data) = serde_json::to_string_pretty(cfg) {
        let _ = fs::write(path, data);
    }
}

// ==========================================
// Обход файлов и фильтрация
// ==========================================
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilterReason {
    SystemDir,
    Extension,
    Excluded,
    NotIncluded,
}

struct FileEntry {
    path: PathBuf,
    rel_path: String,
    status: Result<(), FilterReason>,
}

fn normalize_path(p: &Path) -> String {
    let s = p.components()
        .filter(|c| !matches!(c, std::path::Component::CurDir))
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/");
    if s.is_empty() { ".".to_string() } else { s }
}

fn collect_files(source: &Path, cfg: &Config) -> Vec<FileEntry> {
    let mut entries = Vec::new();
    
    let abs_source = fs::canonicalize(source).unwrap_or_else(|_| source.to_path_buf());

    let mut abs_include_dirs = Vec::new();
    for inc in &cfg.include_dirs {
        let p = Path::new(inc);
        let abs_p = if p.is_absolute() { p.to_path_buf() } else { abs_source.join(p) };
        let canonical_p = fs::canonicalize(&abs_p).unwrap_or(abs_p);
        abs_include_dirs.push(canonical_p);
    }

    let mut abs_exclude_dirs = Vec::new();
    for excl in &cfg.exclude_paths {
        let p = Path::new(excl);
        let abs_p = if p.is_absolute() { p.to_path_buf() } else { abs_source.join(p) };
        let canonical_p = fs::canonicalize(&abs_p).unwrap_or(abs_p);
        abs_exclude_dirs.push(canonical_p);
    }

    let mut it = WalkDir::new(source).into_iter();

    while let Some(Ok(entry)) = it.next() {
        let is_dir = entry.file_type().is_dir();
        let name = entry.file_name().to_string_lossy();
        let entry_path = entry.path();
        
        let rel_path_raw = entry_path.strip_prefix(source).unwrap_or(entry_path);
        let rel_path = normalize_path(rel_path_raw);
        let rel_path_display = if is_dir && rel_path != "." && !rel_path.ends_with('/') {
            rel_path.clone() + "/"
        } else {
            rel_path.clone()
        };

        let abs_entry = fs::canonicalize(entry_path).unwrap_or_else(|_| entry_path.to_path_buf());

        // 1. System Dirs Filter
        if cfg.ignore_system_dirs && is_dir && (name.starts_with('.') || name == "target" || name == "node_modules") {
            entries.push(FileEntry {
                path: entry_path.to_path_buf(),
                rel_path: rel_path_display,
                status: Err(FilterReason::SystemDir),
            });
            it.skip_current_dir();
            continue;
        }

        // 2. Exclude Paths Filter (Directories & Files)
        let mut is_excluded = false;
        for excl in &abs_exclude_dirs {
            if abs_entry.starts_with(excl) {
                is_excluded = true;
                break;
            }
        }
        
        if is_excluded {
            entries.push(FileEntry {
                path: entry_path.to_path_buf(),
                rel_path: rel_path_display,
                status: Err(FilterReason::Excluded),
            });
            if is_dir {
                it.skip_current_dir();
            }
            continue;
        }

        // 3. Include Dirs Filter
        if !abs_include_dirs.is_empty() {
            let mut is_included = false;
            let mut can_reach_included = false;

            for inc in &abs_include_dirs {
                if !is_dir {
                    if abs_entry.starts_with(inc) {
                        is_included = true;
                        break;
                    }
                } else {
                    if abs_entry == *inc || inc.starts_with(&abs_entry) || abs_entry.starts_with(inc) {
                        can_reach_included = true;
                    }
                }
            }

            if is_dir {
                if !can_reach_included {
                    entries.push(FileEntry {
                        path: entry_path.to_path_buf(),
                        rel_path: rel_path_display,
                        status: Err(FilterReason::NotIncluded),
                    });
                    it.skip_current_dir();
                    continue;
                }
            } else {
                if !is_included {
                    entries.push(FileEntry {
                        path: entry_path.to_path_buf(),
                        rel_path: rel_path_display,
                        status: Err(FilterReason::NotIncluded),
                    });
                    continue;
                }
            }
        }

        // 4. Files: Extensions
        if entry.file_type().is_file() {
            if !cfg.use_all_files {
                if let Some(ext) = entry_path.extension() {
                    let ext_str = format!(".{}", ext.to_string_lossy());
                    if !cfg.enabled_extensions.contains(&ext_str) {
                        entries.push(FileEntry {
                            path: entry_path.to_path_buf(),
                            rel_path: rel_path_display,
                            status: Err(FilterReason::Extension),
                        });
                        continue;
                    }
                } else {
                    entries.push(FileEntry {
                        path: entry_path.to_path_buf(),
                        rel_path: rel_path_display,
                        status: Err(FilterReason::Extension),
                    });
                    continue;
                }
            }

            entries.push(FileEntry {
                path: entry_path.to_path_buf(),
                rel_path: rel_path_display,
                status: Ok(())
            });
        }
    }

    entries.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    entries
}

// ==========================================
// Превью и Копирование
// ==========================================
fn print_preview(entries: &[FileEntry], _cfg: &Config, lang: Lang, interactive: bool) -> bool {
    let ok_entries: Vec<&FileEntry> = entries.iter().filter(|e| e.status.is_ok()).collect();
    let total_ok = ok_entries.len();

    println!("\n[{}]", tr(Msg::PreviewTitle, lang).replace("{}", &total_ok.to_string()));

    let mut count = 0;
    let mut page_count = 0;

    for entry in entries {
        count += 1;
        let marker = match entry.status {
            Ok(_) => tr(Msg::StatusOk, lang),
            Err(FilterReason::Excluded) => tr(Msg::StatusExcl, lang),
            Err(FilterReason::SystemDir) => tr(Msg::StatusSys, lang),
            Err(FilterReason::Extension) => tr(Msg::StatusExt, lang),
            Err(FilterReason::NotIncluded) => tr(Msg::StatusNotIncl, lang),
        };
        println!("{:4}. {} {}", count, entry.rel_path, marker);
        page_count += 1;

        if interactive && page_count >= 50 {
            println!("\n{}", tr(Msg::PreviewNext, lang));
            let mut input = String::new();
            let _ = io::stdin().read_line(&mut input);
            if input.trim().eq_ignore_ascii_case("q") {
                return false;
            }
            page_count = 0;
        }
    }

    if interactive && total_ok > 0 {
        println!("\n{}", tr(Msg::RunCopyPrompt, lang));
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        if input.trim().eq_ignore_ascii_case(&tr(Msg::Yes, lang)) || input.trim().eq_ignore_ascii_case("y") {
            return true;
        }
    }
    false
}

fn copy_files(entries: &[FileEntry], cfg: &Config, lang: Lang) {
    let ok_entries: Vec<&FileEntry> = entries.iter().filter(|e| e.status.is_ok()).collect();
    if ok_entries.is_empty() {
        println!("No valid files to copy.");
        return;
    }

    let out_path = PathBuf::from(&cfg.last_output);
    let file = match fs::File::create(&out_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error creating output file: {}", e);
            return;
        }
    };
    let mut writer = BufWriter::new(file);

    for (i, entry) in ok_entries.iter().enumerate() {
        let mut content = match fs::read_to_string(&entry.path) {
            Ok(c) => c,
            Err(_) => {
                eprintln!("{}: {}", tr(Msg::Utf8Warn, lang), entry.rel_path);
                continue;
            }
        };

        if content.ends_with('\n') { content.pop(); }

        let _ = writeln!(writer, "{}", entry.rel_path);
        let _ = writeln!(writer, "{}", content);

        if i < ok_entries.len() - 1 {
            for _ in 0..cfg.separator_lines {
                let _ = writeln!(writer);
            }
        }
    }
    let _ = writer.flush();
    println!("Done! Copied {} files to {}", ok_entries.len(), cfg.last_output);
}

// ==========================================
// CLI и Интерактивное меню
// ==========================================
#[derive(Parser, Debug)]
#[command(name = "copy_prog", version = "1.0", author = "M. Mikh. Mir")]
struct Cli {
    #[arg(long, default_value = ".")]
    source: String,
    #[arg(long, default_value = "merged_output.txt")]
    output: String,
    #[arg(long, default_value_t = 2)]
    sep: u8,
    #[arg(long, value_delimiter = ',')]
    exts: Option<Vec<String>>,
    #[arg(long)]
    all_files: bool,
    #[arg(long)]
    ignore_system_dirs: bool,
    #[arg(long)]
    include_system_dirs: bool,
    #[arg(long, value_delimiter = ',')]
    exclude: Option<Vec<String>>,
    #[arg(long, value_delimiter = ',')]
    only_dirs: Option<Vec<String>>,
    #[arg(long, default_value = "RU")]
    lang: String,
    #[arg(short, long)]
    preview: bool,
    #[arg(long)]
    reset_config: bool,
    #[arg(long)]
    save_config: bool,
    #[arg(long)]
    usage_details: bool,
}

fn run_interactive(cfg: &mut Config) {
    loop {
        let lang = cfg.language.parse::<Lang>().unwrap_or(Lang::RU);
        println!("\n========================================");
        println!("{}", tr(Msg::MenuTitle, lang));
        println!("1. {} ({})", tr(Msg::M1, lang), cfg.last_source);

        let incl_text = if cfg.include_dirs.is_empty() {
            match lang { Lang::RU => "Все папки", Lang::EN => "All folders", Lang::ZH => "所有文件夹" }.to_string()
        } else {
            match lang {
                Lang::RU => format!("{} папок", cfg.include_dirs.len()),
                Lang::EN => format!("{} folders", cfg.include_dirs.len()),
                Lang::ZH => format!("{} 个文件夹", cfg.include_dirs.len()),
            }
        };
        println!("2. {} ({})", tr(Msg::M2, lang), incl_text);
        println!("3. {} ({:?})", tr(Msg::M3, lang), cfg.enabled_extensions);
        println!("4. {} ({})", tr(Msg::M4, lang), cfg.separator_lines);
        println!("5. {} ({})", tr(Msg::M5, lang), cfg.ignore_system_dirs);

        let excl_text = if cfg.exclude_paths.is_empty() {
            match lang { Lang::RU => "Нет", Lang::EN => "None", Lang::ZH => "无" }.to_string()
        } else {
            match lang {
                Lang::RU => format!("{} путей", cfg.exclude_paths.len()),
                Lang::EN => format!("{} paths", cfg.exclude_paths.len()),
                Lang::ZH => format!("{} 个路径", cfg.exclude_paths.len()),
            }
        };
        println!("6. {} ({})", tr(Msg::M6, lang), excl_text);
        println!("7. {} ({})", tr(Msg::M7, lang), cfg.last_output);
        println!("8. {}", tr(Msg::M8, lang));
        println!("9. {}", tr(Msg::M9, lang));
        println!("10. {}", tr(Msg::M10, lang));
        println!("11. {}", tr(Msg::M11, lang));
        println!("12. {}", tr(Msg::M12, lang));
        println!("13. {}", tr(Msg::M13, lang));
        println!("14. {}", tr(Msg::M14, lang));
        println!("========================================");
        println!("{}", tr(Msg::Footer, lang));
        print!("{}", tr(Msg::PromptChoice, lang));
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        let _ = io::stdin().read_line(&mut choice);
        let choice = choice.trim();

        match choice {
            "1" => {
                print!("{}", tr(Msg::PromptSource, lang));
                io::stdout().flush().unwrap();
                let mut s = String::new(); let _ = io::stdin().read_line(&mut s);
                cfg.last_source = s.trim().to_string();
            }
            "2" => {
                println!("{}", tr(Msg::CurrentIncl, lang));
                for e in &cfg.include_dirs { println!("  - {}", e); }
                let mut sub = String::new(); let _ = io::stdin().read_line(&mut sub);
                match sub.trim() {
                    "1" => {
                        print!("{}", tr(Msg::PromptAddIncl, lang));
                        io::stdout().flush().unwrap();
                        let mut p = String::new(); let _ = io::stdin().read_line(&mut p);
                        let p_trim = p.trim();
                        if !p_trim.is_empty() { cfg.include_dirs.push(p_trim.to_string()); }
                    }
                    "2" => {
                        print!("{}", tr(Msg::PromptRemIncl, lang));
                        io::stdout().flush().unwrap();
                        let mut p = String::new(); let _ = io::stdin().read_line(&mut p);
                        cfg.include_dirs.retain(|e| e != p.trim());
                    }
                    "3" => { cfg.include_dirs.clear(); println!("{}", tr(Msg::PromptClearIncl, lang)); }
                    _ => {}
                }
            }
            "3" => {
                println!("{}", tr(Msg::CurrentExts, lang));
                let mut sub = String::new(); let _ = io::stdin().read_line(&mut sub);
                match sub.trim() {
                    "1" => {
                        print!("{}", tr(Msg::PromptAddExt, lang));
                        io::stdout().flush().unwrap();
                        let mut ext = String::new(); let _ = io::stdin().read_line(&mut ext);
                        let ext = ext.trim();
                        if !ext.starts_with('.') { cfg.enabled_extensions.push(format!(".{}", ext)); }
                        else { cfg.enabled_extensions.push(ext.to_string()); }
                    }
                    "2" => {
                        print!("{}", tr(Msg::PromptRemExt, lang));
                        io::stdout().flush().unwrap();
                        let mut ext = String::new(); let _ = io::stdin().read_line(&mut ext);
                        let ext = ext.trim();
                        cfg.enabled_extensions.retain(|e| e != ext && e != &format!(".{}", ext));
                    }
                    "3" => { cfg.use_all_files = !cfg.use_all_files; println!("{}", tr(Msg::UseAllFiles, lang)); }
                    _ => {}
                }
            }
            "4" => {
                print!("{}", tr(Msg::PromptSep, lang));
                io::stdout().flush().unwrap();
                let mut s = String::new(); let _ = io::stdin().read_line(&mut s);
                if let Ok(n) = s.trim().parse::<u8>() { if n <= 5 { cfg.separator_lines = n; } }
            }
            "5" => cfg.ignore_system_dirs = !cfg.ignore_system_dirs,
            "6" => {
                println!("{}", tr(Msg::CurrentExcl, lang));
                for e in &cfg.exclude_paths { println!("  - {}", e); }
                let mut sub = String::new(); let _ = io::stdin().read_line(&mut sub);
                match sub.trim() {
                    "1" => {
                        print!("{}", tr(Msg::PromptAddExcl, lang));
                        io::stdout().flush().unwrap();
                        let mut p = String::new(); let _ = io::stdin().read_line(&mut p);
                        cfg.exclude_paths.push(p.trim().to_string());
                    }
                    "2" => {
                        print!("{}", tr(Msg::PromptRemExcl, lang));
                        io::stdout().flush().unwrap();
                        let mut p = String::new(); let _ = io::stdin().read_line(&mut p);
                        cfg.exclude_paths.retain(|e| e != p.trim());
                    }
                    "3" => { cfg.exclude_paths.clear(); println!("{}", tr(Msg::PromptClearExcl, lang)); }
                    _ => {}
                }
            }
            "7" => {
                print!("{}", tr(Msg::PromptOutput, lang));
                io::stdout().flush().unwrap();
                let mut s = String::new(); let _ = io::stdin().read_line(&mut s);
                cfg.last_output = s.trim().to_string();
            }
            "8" => {
                let source_path = PathBuf::from(&cfg.last_source);
                let files = collect_files(&source_path, cfg);
                copy_files(&files, cfg, lang);
            }
            "9" => {
                let source_path = PathBuf::from(&cfg.last_source);
                let files = collect_files(&source_path, cfg);
                if print_preview(&files, cfg, lang, true) {
                    copy_files(&files, cfg, lang);
                }
            }
            "10" => { *cfg = Config::default(); println!("{}", tr(Msg::ResetSuccess, lang)); }
            "11" => { save_config(cfg); println!("{}", tr(Msg::SaveSuccess, lang)); }
            "12" => println!("{}", tr(Msg::UsageDetails, lang)),
            "13" => {
                println!("1. RU\n2. EN\n3. ZH");
                let mut s = String::new(); let _ = io::stdin().read_line(&mut s);
                match s.trim() {
                    "1" | "RU" | "Рус" => cfg.language = "RU".into(),
                    "2" | "EN" | "Eng" => cfg.language = "EN".into(),
                    "3" | "ZH" | "中文" => cfg.language = "ZH".into(),
                    _ => {}
                }
            }
            "14" => { println!("{}", tr(Msg::ExitMsg, lang)); break; }
            _ => println!("{}", tr(Msg::InvalidChoice, lang)),
        }
    }
}

fn main() {
    setup_windows_console();

    let is_cli = std::env::args().len() > 1;
    let mut cfg = load_config();

    if is_cli {
        let cli = Cli::parse();
        let lang = cli.lang.parse::<Lang>().unwrap_or_else(|_| {
            eprintln!("Invalid language. Use RU, EN, or ZH.");
            process::exit(1);
        });

        if cli.reset_config {
            cfg = Config::default();
            save_config(&cfg);
            println!("{}", tr(Msg::ResetSuccess, lang));
            return;
        }

        if cli.usage_details {
            println!("{}", tr(Msg::UsageDetails, lang));
            return;
        }

        cfg.last_source = cli.source;
        cfg.last_output = cli.output;
        cfg.separator_lines = cli.sep;
        if let Some(exts) = cli.exts { cfg.enabled_extensions = exts; }
        if cli.all_files { cfg.use_all_files = true; }
        if cli.include_system_dirs { cfg.ignore_system_dirs = false; }
        else if cli.ignore_system_dirs { cfg.ignore_system_dirs = true; }
        if let Some(excl) = cli.exclude { cfg.exclude_paths = excl; }
        if let Some(dirs) = cli.only_dirs { cfg.include_dirs = dirs; }

        if cli.save_config {
            save_config(&cfg);
            println!("{}", tr(Msg::SaveSuccess, lang));
            return;
        }

        let source_path = PathBuf::from(&cfg.last_source);
        if !source_path.exists() {
            eprintln!("Source path does not exist: {}", cfg.last_source);
            process::exit(1);
        }

        let files = collect_files(&source_path, &cfg);

        if cli.preview {
            print_preview(&files, &cfg, lang, false);
            return;
        }

        copy_files(&files, &cfg, lang);

    } else {
        run_interactive(&mut cfg);
    }
}