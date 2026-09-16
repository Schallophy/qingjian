//! 设置窗口根组件：左侧导航 + 右侧当前分节页。每改一项就原地写回 `config.toml`（保留注释）再重读，
//! 界面始终反映文件内容；Server 每秒看 mtime 热加载。
//! 状态在这里，消息在 [`message`]，生命周期在 [`component`]，表单零件在 [`controls`]，各页在 [`pages`]。

mod cloud_status;
mod component;
mod controls;
mod message;
mod pages;

use std::path::{Path, PathBuf};

use qingjian_platform::Config;
use windows_reactor::*;

use self::cloud_status::CloudStatus;
pub(crate) use self::message::Message;
use self::pages::{
    about, advanced, candidates, cloud, dictionaries, fuzzy, general, phrases, shortcut, usage,
};

/// 左侧标签固定宽度，让各行控件对齐。
const LABEL_WIDTH: f64 = 220.0;

/// 设置窗口状态。
pub(crate) struct Settings {
    /// 当前配置，每次改动后从盘上重读。
    pub(super) config: Config,

    /// `config.toml` 路径。
    path: PathBuf,

    /// 当前导航分节 tag。
    page: String,

    /// 云服务「测试连接」的状态。
    cloud_status: CloudStatus,

    /// 系统里的字族名（DirectWrite），「字体」框的提示用。
    families: Vec<String>,

    /// 「字体」框里正在敲的文字；`None` 显示配置里的值。
    font_query: Option<String>,

    /// 「自定义短语」页：新规则的输入码 / 文本 / 候选位置，与最近一次保存的错误。
    pub(super) phrase_code: String,
    pub(super) phrase_text: String,
    pub(super) phrase_position: usize,
    pub(super) phrase_error: String,
}

impl Settings {
    /// `%APPDATA%\Qingjian\config.toml`；取不到 `APPDATA` 退回工作目录。
    fn config_path() -> PathBuf {
        std::env::var_os("APPDATA")
            .map(|dir| PathBuf::from(dir).join("Qingjian").join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("config.toml"))
    }

    /// 数据目录 `%APPDATA%\Qingjian`。
    fn data_dir(&self) -> &Path {
        self.path.parent().unwrap_or_else(|| Path::new("."))
    }

    /// 落盘一个配置值再重读。失败只打印。
    fn save(&mut self, section: &str, key: &str, value: impl Into<toml_edit::Value>) {
        if let Err(error) = Config::set_value(&self.path, section, key, value) {
            eprintln!("保存 [{section}] {key} 失败: {error}");
            return;
        }
        self.reload();
    }

    /// 落盘一个字符串数组再重读。
    fn save_array(&mut self, section: &str, key: &str, values: &[String]) {
        if let Err(error) = Config::set_array(&self.path, section, key, values) {
            eprintln!("保存 [{section}] {key} 失败: {error}");
            return;
        }
        self.reload();
    }

    fn reload(&mut self) {
        if let Ok(config) = Config::load(&self.path) {
            self.config = config;
        }
    }

    /// 落盘自定义短语列表；校验失败把错误留在页上，不改文件。
    fn save_phrases(&mut self, phrases: &[qingjian_core::CustomPhrase]) {
        match Config::set_custom_phrases(&self.path, phrases) {
            Ok(()) => {
                self.phrase_error.clear();
                self.reload();
            }
            Err(error) => self.phrase_error = error,
        }
    }

    /// 把「添加规则」表单里的值追加成一条短语；失败（输入码不合法 / 位置被占）显示错误。
    fn add_phrase(&mut self) {
        let mut phrases = self.config.custom_phrases.clone();
        phrases.push(qingjian_core::CustomPhrase {
            code: self.phrase_code.trim().to_owned(),
            text: self.phrase_text.clone(),
            position: self.phrase_position,
            enabled: true,
        });
        match Config::set_custom_phrases(&self.path, &phrases) {
            Ok(()) => {
                self.phrase_code.clear();
                self.phrase_text.clear();
                self.phrase_error.clear();
                self.reload();
            }
            Err(error) => self.phrase_error = error,
        }
    }

    fn page_content(&self, context: &mut ViewContext<Self>) -> View {
        match self.page.as_str() {
            "candidates" => candidates::view(self, context),
            "shortcut" => shortcut::view(self, context),
            "cloud" => cloud::view(self, context),
            "fuzzy" => fuzzy::view(self, context),
            "dictionaries" => dictionaries::view(self, context),
            "phrases" => phrases::view(self, context),
            "usage" => usage::view(self, context),
            "advanced" => advanced::view(self, context),
            "about" => about::view(self, context),
            _ => general::view(self, context),
        }
    }
}
