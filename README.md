# lyrics-next

在终端下为 mpris 提供歌词.

## 安装

```sh
cargo install lyrics-next
```

文件路径为 `~/.lyrics/`

终端歌词显示，使用 mpris 获取播放信息，自动下载歌词。

**KeyMap**

key            | action 
--------------:|------
`h` / `?`      | 帮助
`q` / `ESC`    | 退出
`d` / `delete` | 删除歌词
`left`         | 后退
`right`        | 前进
`space`        | 暂停/播放
`n`            | 下一曲
`p`            | 上一曲
`s`            | 搜索,手动更新
`t`            | 切换标题显示
`c`            | 歌词居中


> player 需要支持 mpris track_id 才可以控制歌曲播放。

**Search key**

key            | action 
--------------:|------
`q` / `ESC`    | 退出到歌词界面.
`h` / `?`      | 帮助.
`n` / `Down`   |下一个
`p` / `Up`     |上一个
`l` / `Enter`  |下载

## 配置

配置文件 `~/.lyrics/lyrics.toml`

- player-filter 设置过滤黑名单和白名单
- ui 设置显示区域
- sources 设置使用的所搜索源

```toml
[player-filter]
except = ["browser", "video", "screen-cast", "chromium", "firefox"]
only = []

[ui]
title = true
time = false
progress_bar = true
text_center = false

[sources]
netease = true
qq = true
kugou = true
```
