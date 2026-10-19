<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="../assets/icons/aeon_logo.svg">
    <source media="(prefers-color-scheme: light)" srcset="../assets/icons/aeon_logo_dark.svg">
    <img src="../assets/icons/aeon_logo.svg" alt="Aeon Engine Logo" width="160">
  </picture>

  <h1>Aeon Engine</h1>

  <p>
    <a href="https://opensource.org/licenses/MPL-2.0"><img src="https://img.shields.io/badge/License-MPL_2.0-orange.svg" alt="License: MPL 2.0"></a>
    <a href="https://github.com/aethelisdev/aeon-engine"><img src="https://img.shields.io/badge/status-in_active_development-green" alt="Status"></a>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-v1.99.0-orange?logo=rust&logoColor=white" alt="Rust"></a>
    <a href="https://youtube.com/@Aeonengine"><img src="https://img.shields.io/badge/AeonEngine-FF0000?style=flat&logo=youtube&logoColor=white" alt="YouTube"></a>
    <a href="https://instagram.com/aeonengine"><img src="https://img.shields.io/badge/AeonEngine-E4405F?style=flat&logo=instagram&logoColor=white" alt="Instagram"></a>
    <a href="https://x.com/aeonengine"><img src="https://img.shields.io/badge/AeonEngine-000000?style=flat&logo=x&logoColor=white" alt="X / Twitter"></a>
  </p>

  <p>
    <a href="../README.md">English</a> | <a href="README_tr.md">Türkçe</a> | <b>日本語</b> | <a href="README_zh.md">简体中文</a>
  </p>
</div>

![Aeon Engine Play Mode](../assets/screenshots/aeonengineplaymode.png)

## Aeon Engine とは？
ほぼ完全に Safe Rust で書かれた、完全モジュール型のゲームエンジンです。

## 開発の背景
Rust の安全性とパフォーマンスに魅力を感じ、自身のゲーム開発のために軽量でモジュール性の高いエンジンを求めていました。既存のエンジンは要件に対して重量級すぎる一方、軽量なエンジンはビジュアル面で満足のいくものではありませんでした。そのため、高いグラフィック品質と軽量性を両立したエンジンを目指し、独自開発をスタートしました。

## Aeon Engine の目標
開発プロセスにおいて設定している主な目標は以下の通りです：

- **モジュール性（Modularity）**: 多くの依存関係に縛られることなく、エンジンのあらゆるサブシステムを容易に切り離し、または交換できる設計。
- **ユーザー環境に適応するシステム**: 初期状態ではゼロ負荷で起動し、レンダリング、物理演算、オーディオなどのモジュールはユーザーの必要に応じて動的に有効化・無効化されます。また、低スペック環境向けに、長期的にはこれらのリソースを RAM から解放できるクリーンアップ機能の導入も検討しています。
- **Rust コードとビジュアルプログラミングの統合（RedWrite）**: RedWrite は、Rust コードとビジュアルプログラミングをリアルタイムに同期させる機能です。どちらか一方で行われた変更が、もう一方へ双方向かつ即座に反映されることを目指しています。
- **Android デバイス上でのエディタ動作**: 高いモジュール性と優れたパフォーマンスを確立した暁には、Android デバイス上で直接ゲーム制作が可能なエディタとして展開し、「Android 上での本格的なゲーム開発は不可能」という先入観を打破することを目指しています。

## 開発状況
Aeon Engine は現在アクティブに開発中です。

コアシステムの大部分はすでに実装されていますが、API や各種機能は今後変更される可能性があります。

## 実行方法
システムに Rust がインストールされていることを確認してください。

```bash
git clone https://github.com/aethelisdev/aeon-engine.git
cd aeon-engine

# Aeon Hub（プロジェクトマネージャー兼ランチャー）を起動
cargo run --release

# またはエディタを直接起動:
cargo run -p ae_engine --release
```

## 設計思想

- **モジュール設計**
- **シンプルなエディタ**
- **ハイパフォーマンス**
- **Safe Rust**
- **過度な複雑さの排除**
- **オープンソース**

## ライセンス

## エディター UI — IrisUI

Aeon Engine のエディターは、Rust でゼロから開発された独自の GPU アクセラレーション対応 UI フレームワーク IrisUI によって動作します。

IrisUI は、マルチ・リテインドツリー（Multi-Retained Tree）アーキテクチャ、細粒度の無効化管理（Invalidation）、Taffy ベースの Flexbox/Grid レイアウト、そして GPU ネイティブな SDF（符号付き距離場）レンダリングを組み合わせています。

### 主な機能
- **マルチ・リテインドツリーアーキテクチャ**： 永続的な UI ツリー、保持されるウィジェットの状態、細粒度の変更追跡。
- **インクリメンタルレイアウト**： Taffy を統合し、Flexbox と Grid に対応。キャッシュを利用して UI ツリーを同期します。
- **GPU ネイティブ SDF レンダリング**： インスタンシングされたクワッド（Instanced Quads）を使用して、角丸矩形、ボーダー、シャドウを描画します。
- **バッチレンダリング**： 連続する UI プリミティブをまとめ、描画コマンドを統合します。
- **独自のドッキングシステム（iris-dock）**： ドッキング可能なパネル、タブレイアウト、ドラッグ＆ドロップによるパネル配置の変更、独立したドッキングツリーに対応します。
- **マルチビューポート対応**： 独立したビューポートをサポートし、対応プラットフォームでは独立したネイティブウィンドウも利用できます。
- **テキストレンダリング**： cosmic-text と glyphon を使用し、テキストシェーピングと GPU によるグリフ描画を実現します。

IrisUI は Aeon Engine のエディター全体を支えており、シーン階層、インスペクター、アセットブラウザー、パフォーマンスパネル、エディターウィンドウなどに使用されています。

Aeon Engine のソースコードは **Mozilla Public License 2.0 (MPL 2.0)** の下で公開されています。

- **インディー開発者およびクリエイター向け**: Aeon Engine を使用して商用のプロプライエタリ（ソースコード非公開）ゲームを制作・販売することができます。ゲームロジックやソースコードを公開する義務はありません。
- **エンジンの改変**: コアエンジンのファイル を変更した場合は、その変更分を MPL 2.0 に基づいて公開する必要があります。
- **商用およびエンタープライズライセンス**: 個別の商用ライセンス、プロプライエタリなエンジンフォーク、または企業向けサポートオプションについては、AethelisDEV までお問い合わせください。