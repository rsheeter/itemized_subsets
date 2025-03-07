# Itemization
Exploratory hacking around identification of itemization for an arbitrary text string
aimed at producing subset Google Fonts requests.

```shell
# 世界 - world in Japanese per Translate
# ❤️‍🔥 - a multicodepoint zwj sequence, https://emojipedia.org/heart-on-fire#technical
# Where ~/oss/fonts is a clone of https://github.com/google/fonts
$ cargo run -- --text "Hello 世界 ❤️‍🔥" --fonts-dir ~/oss/fonts
```