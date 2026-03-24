# pocogit

狭いターミナルペイン向けの軽量 git TUI。status / stage / commit / push を素早く回せる。

## Install

### Homebrew (macOS)

```
brew install rc-code-jp/tap/pocogit
```

### Cargo

```
cargo install --path .
```

## Update

### Homebrew

```
brew upgrade pocogit
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
