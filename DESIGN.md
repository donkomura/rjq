# rjq Design Document

## Overview
rjqは、jaqクエリエンジンを使用したインタラクティブなJSONビューアです。ターミナル上でリアルタイムにJSONデータをクエリし、結果を確認できます。

## Architecture

### Core Components

#### AppError
カスタムエラー型（thiserrorクレートを使用）
- JSON解析エラー
- クエリコンパイルエラー
- クエリ実行エラー
- ターミナルエラー

#### App Structure
- `AppConfig`: アプリケーション設定（プロンプト等）
- `AppState`: アプリケーション状態（入力、終了フラグ、エラー状態）
- `JsonData`: JSONデータとクエリ実行ロジック

#### Query Execution
- `QueryResult`: クエリ結果の表現（Single/Multiple/Empty）
- リアルタイムクエリ実行とフォーマット

### Design Principles
1. 関心の分離: 設定、状態、データを明確に分離
2. エラーハンドリング: 適切なエラー型と処理
3. テスタビリティ: ユニットテストが容易な構造
4. パフォーマンス: 必要時のみクエリ実行