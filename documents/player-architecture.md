# Player 模块架构说明

## 概述

本项目已重构为支持 MPD 和 MPRIS 两种播放器协议，并实现了自动回退机制。

## 模块结构

```
src/
├── player/              # 播放器模块
│   ├── mod.rs          # 统一接口和自动回退逻辑
│   ├── mpd.rs          # MPD 协议实现
│   └── mpris.rs        # MPRIS 协议实现
├── song.rs             # 歌曲信息、播放时间和歌词解析
├── config.rs           # 配置管理（包含协议选择）
└── error.rs            # 错误处理
```

## 核心设计

### 1. Player Trait

定义了播放器的统一接口：

```rust
pub trait Player {
    fn get_current_song(&self) -> impl Future<Output = Result<SongInfo, LyricsError>>;
    fn get_position(&self) -> impl Future<Output = Result<f64, LyricsError>>;
    fn player_action(&self, action: PlayerAction, song: &SongInfo) -> 
        impl Future<Output = Result<(), LyricsError>>;
}
```

### 2. 协议选择策略

在 `player/mod.rs` 中实现了三种协议选择模式：

- **Auto**（默认）：优先尝试 MPD，失败后自动回退到 MPRIS
- **Mpd**：仅使用 MPD 协议
- **Mpris**：仅使用 MPRIS 协议

### 3. 自动回退机制

```rust
pub async fn get_current_song() -> Result<SongInfo, LyricsError> {
    let protocol = {
        let config = crate::config::get_config().read().unwrap();
        config.player_filter.protocol
    };

    match protocol {
        PlayerProtocol::Auto => {
            // 优先尝试 MPD
            match MpdPlayer.get_current_song().await {
                Ok(song) => Ok(song),
                Err(_) => {
                    // MPD 失败，回退到 MPRIS
                    MprisPlayer.get_current_song().await
                }
            }
        }
        // ...
    }
}
```

## 配置

### 配置文件示例

```toml
[player-filter]
protocol = "auto"  # 可选值: auto, mpd, mpris
mpd-host = "127.0.0.1"
mpd-port = 6600
only = []          # 白名单
except = []        # 黑名单
```

### PlayerProtocol 枚举

```rust
pub enum PlayerProtocol {
    #[default]
    Auto,    # 自动选择（MPD -> MPRIS）
    Mpris,   # 仅 MPRIS
    Mpd,     # 仅 MPD
}
```

## MPD 实现

### 连接管理

```rust
fn get_client() -> Result<Client, LyricsError> {
    let config = get_config().read().unwrap();
    let addr = format!("{}:{}", 
        config.player_filter.mpd_host,
        config.player_filter.mpd_port
    );
    Client::connect(&addr)?
}
```

### 特性

- 支持歌曲 ID 追踪
- 支持播放控制（播放/暂停、上下曲、快进快退）
- 从 MPD tags 中获取专辑信息

## MPRIS 实现

### 播放器过滤

支持白名单和黑名单机制：

```rust
fn is_valid_player(player: &Player) -> bool {
    let identity = player.identity().to_lowercase();
    
    // 黑名单过滤
    if !config.except.is_empty() && 
       config.except.iter().any(|k| identity.contains(k)) {
        return false;
    }
    
    // 白名单检查
    if !config.only.is_empty() {
        return config.only.iter().any(|k| identity.contains(k));
    }
    
    true
}
```

### 默认黑名单

- browser
- video
- screen-cast
- chromium
- firefox

## UI 改进

### 错误显示优化

修复了错误信息显示位置问题：

- 错误区域从 1 行增加到 3 行
- 将错误显示位置从底部移到顶部（header 之后）
- 使用带边框的块显示错误，提高可见性

```rust
// 布局顺序
let [header_chunk, err_chunk, list_chunk, footer_chunk] = Layout::new(
    Direction::Vertical,
    [
        Constraint::Length(1),    // header
        Constraint::Length(3),    // error (3 行)
        Constraint::Min(3),       // list
        Constraint::Length(1),    // footer
    ],
).areas(area);
```

## 数据结构

### TrackId

支持两种协议的 track ID：

```rust
pub enum TrackId {
    Mpris(String),  # MPRIS track ID
    Mpd(u32),       # MPD song ID
    None,           # 无 track ID
}
```

### SongInfo

歌曲信息结构：

```rust
pub struct SongInfo {
    pub track_id: TrackId,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: f64,
}
```

## 错误处理

所有播放器相关错误都统一转换为 `LyricsError`：

```rust
pub enum LyricsError {
    MprisError(#[from] mpris::DBusError),
    MpdError(#[from] mpd::error::Error),
    NoPlayerFound,
    // ...
}
```

## 使用示例

### 获取当前歌曲

```rust
use crate::song::get_current_song;

let song = get_current_song().await?;
println!("当前播放: {} - {}", song.artist, song.title);
```

### 播放控制

```rust
use crate::song::{player_action, PlayerAction};

// 播放/暂停
player_action(PlayerAction::Toggle, &song).await?;

// 下一首
player_action(PlayerAction::Next, &song).await?;
```

## 最佳实践

1. **优先使用 Auto 模式**：默认配置会自动选择最佳播放器
2. **合理配置过滤规则**：使用白名单/黑名单避免连接到非音乐播放器
3. **错误处理**：所有操作都返回 Result，需要妥善处理错误
4. **避免跨 await 持锁**：在调用异步函数前释放配置锁

## 测试

运行测试：

```bash
cargo test --lib
```

代码质量检查：

```bash
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all --check
```

## 未来改进

1. 添加连接池管理 MPD 连接
2. 实现播放器状态缓存，减少 D-Bus 调用
3. 支持更多播放器协议（如 VLC 的 HTTP API）
4. 添加播放器健康检查机制
