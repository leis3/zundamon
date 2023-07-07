# zundamon

## 使い方

1. リポジトリをクローンする。
```console
$ git clone https://github.com/leis3/zundamon.git
$ cd zundamon
```

2. [ここ](https://github.com/VOICEVOX/voicevox_core#%E7%92%B0%E5%A2%83%E6%A7%8B%E7%AF%89)に従ってVOICEVOXのコアライブラリをダウンロードする。

```console
# Linux x64での例
$ binary=download-linux-x64
$ curl -sSfL https://github.com/VOICEVOX/voicevox_core/releases/latest/download/${binary} -o download
$ chmod +x download
$ ./download # CUDA版を利用する場合は`--device cuda`オプションを付ける
```

3. 環境変数`LD_LIBRARY_PATH`と`DISCORD_TOKEN`を設定して実行する。

```console
$ export LD_LIBRARY_PATH=./voicevox_core:$LD_LIBRARY_PATH
$ export DISCORD_TOKEN=xxxxxx
$ cargo run --release
```
