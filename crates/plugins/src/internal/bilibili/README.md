# Bilibili 音频取流（临时策略说明）

本插件当前采用“仅 MP4 Progressive（durl）”的取流方案，以保证稳定性与简单性。DASH（.mpd/.m4s）解析暂不启用。

## 固定参数（写死）
- `fnval=1`：仅请求 MP4（避免返回 `dash` 字段）
- `platform=html5`：移动端 HTML5，通常返回 MP4，referer 要求更宽松
- `fnver=0`
- `fourk=0`
- `high_quality=1`：开启高画质开关
- `qn=80`：期望 1080P；若接口侧降级，则以返回的 `durl` 为准

参考文档：`other/bilibili-API-collect/docs/video/videostream_url.md`

## 返回与头部
- 仅解析并返回 `data.durl` 的首选条目，不做 DASH 回退
- 返回时附带基础防盗链头：
  - `Referer: https://www.bilibili.com`
  - `User-Agent: Mozilla/5.0 ...`（桌面浏览器 UA）

## 已知限制
- 不支持 4K/HDR/杜比/8K（这些通常需要 DASH 与更高 `fnval` 位）
- 画质以接口实际返回为准；即使 `qn=80`，也可能因资源限制降级

## 后续路线（计划）
- 如需高阶画质与自适应：改为插件返回/合成 `.mpd`，进入统一 DASH 管线
- DASH 管线侧支持分段缓存与按需拉流，避免供应商特例侵入核心层

以上策略为阶段性取舍，旨在尽快恢复可用音频播放能力。
