# pocogit

狭いターミナルペイン向けの軽量 git TUI。status / stage / commit / push を素早く回せる。


https://github.com/user-attachments/assets/e56bad2a-7ee1-4432-9e93-4a5c9e093ec9


## Install

### Nix

```
nix profile install github:rc-code-jp/pocogit#pocogit
```

一時実行する場合は以下を使う。

```
nix run github:rc-code-jp/pocogit#pocogit
```

### Cargo

```
cargo install --path .
```

## Update

### Nix

```
nix profile upgrade pocogit
```

### Cargo

```
cargo install --path . --force
```

## Usage

```
pocogit
```

git リポジトリ内で起動する。

## Keys

| Key | Action |
|---|---|
| `j` / `↓` | 下へ移動 |
| `k` / `↑` | 上へ移動 |
| `s` / `Enter` | stage ⇔ unstage トグル |
| `A` | stage all |
| `U` | unstage all |
| `c` | commit |
| `p` | push |
| `r` | refresh |
| `q` / `Esc` | quit |

マウスクリック・スクロール対応。

## Contributing

コントリビュート歓迎です！詳しくは [CONTRIBUTING.md](CONTRIBUTING.md) をご覧ください。

## License

[MIT](LICENSE)
