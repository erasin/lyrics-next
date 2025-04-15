use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{
        Stylize,
        palette::tailwind::{BLUE, GREEN},
    },
    symbols,
    text::{Line, Span},
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget,
        Widget,
    },
};

use crate::{
    client::{LyricsItem, get_lyrics_client},
    error::LyricsError,
    song::{SongInfo, get_current_song},
};

use super::*;

// search
#[derive(Clone, Default)]
pub(super) struct SearchScreen {
    state: SearchState,
    list_state: ListState,
}

impl SearchScreen {
    // 主渲染函数
    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        // 渲染错误信息
        if let Some(err_msg) = &self.state.error_message {
            render_error(area, buf, err_msg);
            return;
        }

        // 整体垂直布局
        let [header_chunk, list_chunk, footer_chunk] = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Length(1),
            ],
        )
        .areas(area);

        self.render_header(header_chunk, buf);
        self.render_list(list_chunk, buf);
        self.render_footer(footer_chunk, buf);
    }

    pub async fn handle_key_event(&mut self, key_event: &KeyEvent) {
        match key_event.code {
            KeyCode::Char('l') | KeyCode::Enter => self.download().await,
            KeyCode::Up | KeyCode::Char('p') | KeyCode::Char('k') => self.selected_up(),
            KeyCode::Down | KeyCode::Char('n') | KeyCode::Char('j') => self.selected_down(),
            _ => {}
        }
    }

    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Lyric 列表")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("使用 ↓↑ or jk 选择, l or enter 下载")
            .centered()
            .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("搜索").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(LIST_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .state
            .list
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let color = alternate_colors(i);
                Line::from(vec![
                    Span::raw(&item.source).fg(BLUE.c400),
                    Span::raw(" "),
                    Span::raw(&item.title)
                        .fg(YELLOW.c400)
                        .add_modifier(Modifier::BOLD),
                    Span::raw(" "),
                    Span::raw(&item.artist).fg(GREEN.c400),
                    Span::raw(" "),
                    Span::raw(&item.album).add_modifier(Modifier::ITALIC),
                ])
                .bg(color)
                .into()
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.list_state);
    }

    fn selected_up(&mut self) {
        self.list_state.select_previous();
    }

    fn selected_down(&mut self) {
        self.list_state.select_next();
    }

    pub async fn update(&mut self) {
        self.state.update().await;
    }

    async fn download(&mut self) {
        let item_index = self.list_state.selected().unwrap();
        if item_index > self.state.list.len() {
            return;
        }
        self.state.download(item_index).await;
    }

    pub fn lyrics_reset(&mut self) -> bool {
        if self.state.reset_lyrics_cache {
            self.state.reset_lyrics_cache = false;
            return true;
        }
        false
    }
}

#[derive(Clone, Default)]
pub struct SearchState {
    song: SongInfo,
    list: Vec<LyricsItem>,
    /// 新增错误状态
    error_message: Option<String>,
    // 有则重置
    pub reset_lyrics_cache: bool,
}

impl SearchState {
    fn reset(&mut self) {
        *self = Self::default();
    }

    pub async fn update(&mut self) {
        match self.try_update().await {
            Ok(_) => {
                self.error_message = None; // 清除旧错误        
            }
            Err(e) => self.error_message = Some(e.to_string()),
        }
    }

    pub async fn try_update(&mut self) -> Result<(), LyricsError> {
        // 获取当前播放器和歌曲信息
        let song = match get_current_song() {
            Ok(s) => s,
            Err(LyricsError::NoPlayerFound) => {
                self.reset();
                return Ok(());
            }
            Err(e) => return Err(e),
        };

        // 歌曲发生变化时重新加载歌词
        if song != self.song {
            self.reset();
            self.song = song.clone();
            self.list = get_lyrics_client().get_search(&song).await?;
        }

        Ok(())
    }

    pub async fn download(&mut self, item_index: usize) {
        let item = match self.list.get(item_index) {
            Some(i) => i,
            None => {
                self.reset_lyrics_cache = false;
                self.error_message = Some("选择错误，超出范围！".to_string());
                return;
            }
        };

        match get_lyrics_client().download(&self.song, item).await {
            Ok(_) => self.reset_lyrics_cache = true,
            Err(e) => {
                self.error_message = Some(e.to_string());
            }
        }
    }
}
