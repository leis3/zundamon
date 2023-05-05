# zundamon

## 使い方

1. [ここ](https://github.com/VOICEVOX/voicevox_core#%E7%92%B0%E5%A2%83%E6%A7%8B%E7%AF%89)に従ってVOICEVOXのコアライブラリをダウンロードする。

```bash
# Linux x64での例
$ binary=download-linux-x64
$ curl -sSfL https://github.com/VOICEVOX/voicevox_core/releases/latest/download/${binary} -o download
$ chmod +x download
$ ./download # CUDA版を利用する場合は`--device cuda`オプションを付ける
```

2. 実行する。

```bash
$ LD_LIBRARY_PATH=./voicevox_core:$LD_LIBRARY_PATH \
DISCORD_TOKEN=xxxxxx \
cargo run --release
```
