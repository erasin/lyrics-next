# lyric-next

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

**Search key**

key            | action 
--------------:|------
`q` / `ESC`    | 退出到歌词界面.
`h` / `?`      | 帮助.
`n` / `Down`   |下一个
`p` / `Up`     |上一个
`l` / `Enter`  |上一个


## 配置

配置文件 `~/.lyrics/lyrics.toml`

```toml
[ui]
title = true
time = false
progress_bar = true

[sources]
netease = true
qq = true
kugou = true
```
