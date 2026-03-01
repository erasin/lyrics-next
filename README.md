# lyrics-next

在终端下为 MPD 和 MPRIS 播放器提供歌词展示以及搜索功能。

![Screenshot from 2025-06-11 21-05-21](https://github.com/user-attachments/assets/c17ea748-50b0-4a0c-98cb-716e02733fdb)

## 特性

- 🎵 **双协议支持**：同时支持 MPD 和 MPRIS 播放器协议
- 🔄 **智能回退**：自动检测可用播放器，MPD 优先，失败自动切换到 MPRIS
- 🔍 **多源搜索**：支持网易云、QQ音乐、酷狗音乐等多个歌词源
- 💾 **本地缓存**：自动缓存已下载的歌词
- 🎨 **美观界面**：基于 Ratatui 的终端用户界面
- ⌨️ **快捷键控制**：完整的键盘快捷键支持

## 安装

```sh
cargo install lyrics-next
```

或从源码安装：

```sh
git clone https://github.com/erasin/lyrics-next.git
cd lyrics-next
cargo install --path .
```

### 目录结构

- `~/.config/lyrics/config.toml` - 配置文件
- `~/.local/share/lyrics` - 歌词缓存目录
- `~/.cache/lyrics` - 日志目录

## 支持的播放器

### MPD (Music Player Daemon)
- MPD 原生支持
- 需要运行 MPD 服务（默认端口 6600）

### MPRIS (Media Player Remote Interfacing Specification)
支持所有实现了 MPRIS D-Bus 接口的播放器，包括但不限于：
- Spotify
- VLC
- NCMpcPP
- Clementine
- Audacious
- 以及更多...

## 快捷键

### 主界面

|            key | action          |
| -------------: | --------------- |
|      `h` / `?` | 帮助            |
|    `q` / `ESC` | 退出            |
| `d` / `delete` | 删除歌词        |
|         `left` | 后退 5 秒       |
|        `right` | 前进 5 秒       |
|        `space` | 暂停/播放       |
|            `n` | 下一曲          |
|            `p` | 上一曲          |
|            `s` | 搜索/手动更新   |
|            `t` | 切换标题显示    |
|            `c` | 歌词居中        |

> 注意：播放器需要支持 track_id 才可以控制歌曲播放进度。

### 搜索界面

|           key | action            |
| ------------: | ----------------- |
|   `q` / `ESC` | 退出到歌词界面    |
|     `h` / `?` | 帮助              |
|  `n` / `Down` | 下一个            |
|    `p` / `Up` | 上一个            |
| `l` / `Enter` | 下载选中歌词      |

## 配置

配置文件位于 `~/.config/lyrics/config.toml`

### 完整配置示例

```toml
[player-filter]
# 播放器协议选择: auto / mpd / mpris
# auto: 自动选择（优先 MPD，失败回退到 MPRIS）
protocol = "auto"

# MPD 连接配置
mpd-host = "127.0.0.1"
mpd-port = 6600

# 播放器过滤（仅对 MPRIS 有效）
# 黑名单：忽略包含这些关键词的播放器
except = ["browser", "video", "screen-cast", "chromium", "firefox"]
# 白名单：仅使用包含这些关键词的播放器（为空表示不过滤）
only = []

[ui]
# 显示设置
title = true           # 显示歌曲标题
time = false           # 显示时间
progress_bar = true    # 显示进度条
text_center = false    # 歌词居中显示

[sources]
# 歌词源设置
netease = true         # 网易云音乐
qq = true              # QQ音乐
kugou = true           # 酷狗音乐
```

### 协议选择说明

| 值     | 说明                                       |
| ------ | ------------------------------------------ |
| `auto` | 自动选择，优先使用 MPD，失败自动回退 MPRIS |
| `mpd`  | 仅使用 MPD 协议                            |
| `mpris`| 仅使用 MPRIS 协议                          |

## 架构

项目采用模块化设计：

```
src/
├── player/        # 播放器抽象层
│   ├── mod.rs    # 统一接口和自动回退
│   ├── mpd.rs    # MPD 协议实现
│   └── mpris.rs  # MPRIS 协议实现
├── song.rs       # 歌曲信息、播放时间、歌词解析
├── client/       # 歌词源 API 客户端
├── cache/        # 歌词缓存管理
├── ui/           # 终端用户界面
└── config.rs     # 配置管理
```

详细架构说明请参考 [player-architecture.md](./documents/player-architecture.md)

## 开发

### 环境要求

- Rust 1.70+
- MPD 服务（可选，用于测试 MPD 功能）
- 支持 MPRIS 的播放器（可选）

### 构建

```sh
cargo build
```

### 测试

```sh
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all --check
```

### 运行

```sh
cargo run
```

## 常见问题

### Q: 为什么找不到播放器？

A: 
1. 确保你的播放器正在运行
2. 如果使用 MPD，检查 `mpd-host` 和 `mpd-port` 配置是否正确
3. 如果使用 MPRIS，确保播放器实现了 MPRIS D-Bus 接口
4. 检查 `except` 配置是否过滤掉了你的播放器

### Q: 如何只使用 MPD？

A: 在配置文件中设置：
```toml
[player-filter]
protocol = "mpd"
```

### Q: 如何排除浏览器中的视频播放？

A: 默认配置已经过滤了常见浏览器和视频播放器，你也可以自定义：
```toml
[player-filter]
except = ["browser", "video", "chromium", "firefox", "vlc"]
```

### Q: 歌词不同步怎么办？

A: 
1. 使用 `s` 键进入搜索界面
2. 选择其他来源的歌词
3. 下载后自动替换

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

MIT License

## 致谢

- [Ratatui](https://github.com/ratatui-org/ratatui) - 终端 UI 框架
- [mpris](https://github.com/Mange/mpris-rs) - MPRIS Rust 库
- [mpd](https://github.com/kstep/rust-mpd) - MPD Rust 客户端
