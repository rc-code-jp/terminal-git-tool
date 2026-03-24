# Contributing

pocogit へのコントリビュートに興味を持っていただきありがとうございます！

## 開発環境のセットアップ

```bash
git clone https://github.com/rc-code-jp/pocogit.git
cd pocogit
cargo build
```

## 開発の流れ

1. このリポジトリを Fork する
2. feature ブランチを作成する (`git checkout -b feature/amazing-feature`)
3. 変更をコミットする (`git commit -m 'Add amazing feature'`)
4. ブランチを Push する (`git push origin feature/amazing-feature`)
5. Pull Request を作成する

## ビルドとテスト

```bash
# ビルド
cargo build

# リリースビルド
cargo build --release

# テスト
cargo test

# lint
cargo clippy
```

## Pull Request のガイドライン

- PR はできるだけ小さく、1つの変更に集中してください
- 既存のコードスタイルに従ってください
- `cargo clippy` で警告が出ないことを確認してください

## バグ報告・機能リクエスト

[Issues](https://github.com/rc-code-jp/pocogit/issues) からお願いします。

- **バグ報告**: 再現手順、期待される動作、実際の動作を記載してください
- **機能リクエスト**: ユースケースと期待される動作を記載してください

## ライセンス

コントリビュートされたコードは [MIT License](LICENSE) のもとで公開されます。
