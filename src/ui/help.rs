use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use crate::ui::SearchScreen;

use super::{HELP_KEY_STYLE, LyricsScreen};

#[derive(Clone, Default)]
pub(super) struct HelpScreen;

impl HelpScreen {
    // 帮助
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let lyric_lines = LyricsScreen::help();
        // search
        let search_lines = SearchScreen::help();
        // help
        let help_lines = vec![("q | ESC ", " 退出到歌词界面.")];

        let chunks = Layout::new(
            Direction::Vertical,
            [
                Constraint::Min(lyric_lines.len() as u16 + 2),
                Constraint::Min(search_lines.len() as u16 + 2),
                Constraint::Min(help_lines.len() as u16 + 2),
            ],
        );
        let [lyric_chunk, search_chunk, help_chunk] = chunks.areas(area);

        help(lyric_lines).render(lyric_chunk, buf);
        help(search_lines).render(search_chunk, buf);
        help(help_lines).render(help_chunk, buf);
    }
}

// 提取的创建行函数
fn help<'a>(lines: Vec<(&'a str, &'a str)>) -> Paragraph<'a> {
    let lines: Vec<Line> = lines
        .into_iter()
        .map(|(key, description)| {
            Line::from(vec![
                Span::styled(key, HELP_KEY_STYLE),
                Span::raw(":"),
                Span::raw(description),
            ])
        })
        .collect();

    Paragraph::new(lines)
        .block(Block::default().title("搜索").borders(Borders::ALL))
        .wrap(Wrap { trim: true })
}
