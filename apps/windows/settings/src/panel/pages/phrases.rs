//! 「自定义短语」页：按输入码把文本固定到候选位置（`[[custom_phrases]]`），与 macOS 的短语页对齐。
//! 输入码是敲的原始字母串，精确匹配整段；同名候选让位，不会出现两个。

use qingjian_core::CustomPhrase;
use windows_reactor::*;

use crate::panel::controls::{field, labeled, note, page};
use crate::panel::{Message, Settings};

/// 候选位置下拉 1–9。
const POSITIONS: [&str; 9] = [
    "第 1 位",
    "第 2 位",
    "第 3 位",
    "第 4 位",
    "第 5 位",
    "第 6 位",
    "第 7 位",
    "第 8 位",
    "第 9 位",
];

fn position_combo(current: usize, callback: Callback<Option<usize>>) -> ComboBox {
    ComboBox::new()
        .items_source(POSITIONS)
        .selected_index(current.saturating_sub(1).min(POSITIONS.len() - 1))
        .on_selection_changed(callback)
}

/// 一行预览：`ye → 也（第 1 位）`。
fn label(phrase: &CustomPhrase) -> String {
    format!(
        "{} → {}（第 {} 位）",
        phrase.code,
        CustomPhrase::preview(&phrase.text, 40),
        phrase.position
    )
}

/// 规则在列表里的稳定键：输入码 + 位置（校验保证不重复）。
fn phrase_key(phrase: &CustomPhrase) -> String {
    format!("{}@{}", phrase.code, phrase.position)
}

fn rule_row(index: usize, phrase: &CustomPhrase, context: &mut ViewContext<Settings>) -> KeyedView {
    let check = CheckBox::new()
        .is_checked(phrase.enabled)
        .on_is_checked_changed(context.callback(move |on| Message::PhraseToggle(index, on)))
        .content(label(phrase));
    let row = StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(12.0)
        .children((
            check,
            Button::new()
                .on_click(context.message(Message::PhraseRemove(index)))
                .content("删除"),
        ));
    KeyedView::new(phrase_key(phrase), row)
}

pub(crate) fn view(settings: &Settings, context: &mut ViewContext<Settings>) -> View {
    let phrases = &settings.config.custom_phrases;
    let list: View = if phrases.is_empty() {
        note("还没有规则。填好下面三项，点「添加」。")
    } else {
        let rows: Vec<KeyedView> = phrases
            .iter()
            .enumerate()
            .map(|(index, phrase)| rule_row(index, phrase, context))
            .collect();
        StackPanel::new().spacing(6.0).keyed_children(rows)
    };
    let error: View = if settings.phrase_error.is_empty() {
        TextBlock::new().text("").into()
    } else {
        note(&settings.phrase_error)
    };
    let body = StackPanel::new().spacing(12.0).children([
        note("输入码按你敲的原始字母串精确匹配（ye、nihao、prompt 都行，1–32 个小写字母）；文本固定放在候选的第几位，上屏的是文本本身。词库里的同名候选会让位，不会出现两个。"),
        list,
        TextBlock::new()
            .text("添加规则")
            .font_weight(FontWeight::SEMI_BOLD)
            .into(),
        field(
            "输入码",
            "",
            TextBox::new()
                .text(settings.phrase_code.clone())
                .on_text_changed(context.callback(Message::PhraseCode)),
        ),
        field(
            "文本",
            "",
            TextBox::new()
                .text(settings.phrase_text.clone())
                .on_text_changed(context.callback(Message::PhraseText)),
        ),
        field(
            "候选位置",
            "",
            position_combo(
                settings.phrase_position,
                context.callback(Message::PhrasePosition),
            ),
        ),
        labeled(
            "",
            StackPanel::new()
                .orientation(Orientation::Horizontal)
                .spacing(12.0)
                .children((
                    Button::new()
                        .on_click(context.message(Message::PhraseAdd))
                        .content("添加"),
                    error,
                )),
        ),
    ]);
    page("自定义短语", body)
}
