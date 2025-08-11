# WebAssembly FFI 型安全性調査レポート

## エグゼクティブサマリー

本ドキュメントは、Hail MaryプロジェクトにおけるWebAssembly向け型安全FFI（Foreign Function Interface）の実装に関する調査結果をまとめたものです。特に、Rust-WASMモジュールとGoベースのWazeroランタイムの統合に焦点を当てています。

**主要な発見**: safer-ffiおよび類似の型安全FFIライブラリは**WebAssemblyターゲットと互換性がありません**。型安全性は、デザインパターンとベストプラクティスを用いて手動で実装する必要があります。

## 調査背景

### 調査依頼内容
`rust-wasm/src/lib.rs`におけるWebAssembly FFI実装の型安全性を向上させるため、safer-ffiの使用可能性を調査する。

### 調査範囲
1. safer-ffiとWebAssemblyの互換性
2. WASM向け代替型安全FFIソリューション
3. WASMにおける`extern "C"`実装のベストプラクティス
4. Wazero固有の考慮事項

## 主要な調査結果

### 1. safer-ffiとWebAssemblyの非互換性

#### safer-ffiが使用できない理由

| 項目 | ネイティブFFI | WebAssembly FFI |
| **実行環境** | OSネイティブ実行 (x86_64, ARM) | WASMランタイムサンドボックス |
| **メモリモデル** | OS管理の共有メモリ | 隔離された線形メモリ |
| **ABI** | システムABI (cdecl, stdcall) | WebAssembly ABI |
| **ポインタ** | 64ビット実アドレス | 32ビットオフセット |
| **ライブラリ形式** | .so/.dll/.dylib | .wasm |
| **実行方法** | OS直接実行 | ランタイム経由 |

**結論**: safer-ffiはネイティブOS環境向けに設計されており、WebAssemblyのサンドボックス実行モデルとは根本的に互換性がありません。

### 2. ライブラリ環境の分析

#### FFIライブラリとWASMサポート

| ライブラリ | 用途 | WASMサポート | 備考 |
|---------|---------|--------------|-------|
| **safer-ffi** | Rust ⇔ C FFI | ❌ | ネイティブ専用、OS ABI必須 |
| **cxx** | Rust ⇔ C++ FFI | ❌ | ネイティブ専用、C++特化 |
| **wasm-bindgen** | Rust ⇔ JavaScript | ⚠️ | ブラウザ/Node.js専用、Wazero非対応 |
| **wit-bindgen** | Component Model | ⚠️ | WazeroはComponent Model未対応 |
| **libc** | C標準ライブラリ | ⚠️ | WASM環境では機能制限あり |

**重要な洞察**: Wazeroランタイム向けのWebAssembly専用型安全FFIライブラリは存在しません。

### 3. WebAssemblyの制約

#### 技術的制限
- **ガベージコレクションなし**: 手動メモリ管理が必要
- **線形メモリモデル**: 32ビットアドレッシング、連続メモリ空間
- **サンドボックス化**: ホストシステムから完全に隔離
- **直接システムコールなし**: すべてのI/Oはランタイム経由
- **シングルスレッド**: 現行仕様では並列処理不可

#### Wazero固有の制約
- **Pure Go実装**: CGO依存なし
- **Component Model未対応**: wit-bindgen使用不可
- **手動メモリ共有**: ホストとゲスト間でallocation/deallocationの調整が必要

## 型安全実装パターン

### 1. エラーコードパターン

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum WasmError {
    Success = 0,
    InvalidInput = 1,
    OutOfMemory = 2,
    InvalidUtf8 = 3,
    NullPointer = 4,
}

#[repr(C)]
pub struct StringResult {
    ptr: *mut c_char,
    error: WasmError,
}

#[no_mangle]
pub extern "C" fn safe_operation(input: *const c_char) -> StringResult {
    if input.is_null() {
        return StringResult {
            ptr: std::ptr::null_mut(),
            error: WasmError::NullPointer,
        };
    }
    
    // Safe processing with explicit error handling
    match process_input(input) {
        Ok(result) => StringResult {
            ptr: result.into_raw(),
            error: WasmError::Success,
        },
        Err(e) => StringResult {
            ptr: std::ptr::null_mut(),
            error: map_error(e),
        },
    }
}
```

### 2. 不透明ハンドルパターン

```rust
use std::marker::PhantomData;

#[repr(C)]
pub struct OpaqueHandle {
    _data: (),
    _marker: PhantomData<(*mut u8, std::marker::PhantomPinned)>,
}

// Type-safe handle wrapper
#[repr(transparent)]
pub struct TypedHandle<T> {
    ptr: *mut c_void,
    _marker: PhantomData<T>,
}

impl<T> TypedHandle<T> {
    fn from_box(value: Box<T>) -> Self {
        TypedHandle {
            ptr: Box::into_raw(value) as *mut c_void,
            _marker: PhantomData,
        }
    }
    
    unsafe fn as_ref(&self) -> Option<&T> {
        (self.ptr as *const T).as_ref()
    }
}

#[no_mangle]
pub extern "C" fn create_handle() -> *mut c_void {
    let data = Box::new(InternalData::new());
    Box::into_raw(data) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn destroy_handle(handle: *mut c_void) {
    if !handle.is_null() {
        let _ = Box::from_raw(handle as *mut InternalData);
    }
}
```

### 3. バリデーション層パターン

```rust
// Internal type-safe implementation
fn safe_process_internal(input: &str) -> Result<String, ProcessError> {
    // Type-safe business logic
    validate_input(input)?;
    transform_data(input)
}

// FFI boundary with validation
#[no_mangle]
pub extern "C" fn process_string(input: *const c_char) -> *mut c_char {
    // 1. Input validation
    let input_str = match validate_c_string(input) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };
    
    // 2. Type-safe processing
    let result = match safe_process_internal(input_str) {
        Ok(r) => r,
        Err(_) => return std::ptr::null_mut(),
    };
    
    // 3. Output conversion
    match CString::new(result) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

fn validate_c_string(ptr: *const c_char) -> Option<&str> {
    if ptr.is_null() {
        return None;
    }
    
    unsafe {
        CStr::from_ptr(ptr).to_str().ok()
    }
}
```

### 4. Wazero固有のメモリ管理

```rust
// Memory allocation for host
#[no_mangle]
pub extern "C" fn allocate(size: usize) -> *mut u8 {
    let mut buffer = Vec::with_capacity(size);
    let ptr = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    ptr
}

// Memory deallocation
#[no_mangle]
pub unsafe extern "C" fn deallocate(ptr: *mut u8, size: usize) {
    if !ptr.is_null() {
        let _ = Vec::from_raw_parts(ptr, 0, size);
    }
}

// Type-safe wrapper with RAII
pub struct WasmBuffer {
    ptr: *mut u8,
    len: usize,
    capacity: usize,
}

impl WasmBuffer {
    fn new(size: usize) -> Self {
        let ptr = allocate(size);
        WasmBuffer {
            ptr,
            len: 0,
            capacity: size,
        }
    }
    
    fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl Drop for WasmBuffer {
    fn drop(&mut self) {
        unsafe { deallocate(self.ptr, self.capacity); }
    }
}
```

## Hail Maryプロジェクトへの実装推奨事項

### 現在の実装分析

`rust-wasm/src/lib.rs`の既存実装はWebAssemblyのベストプラクティスに従っています：

```rust
#[no_mangle]
pub extern "C" fn hello_world() -> *mut c_char {
    let message = CString::new("hello, world").unwrap();
    message.into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    let _ = CString::from_raw(s);
}
```

### 改善された型安全バージョン

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// Error handling
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum HailMaryError {
    Success = 0,
    NullPointer = 1,
    InvalidUtf8 = 2,
    AllocationFailed = 3,
}

// Result wrapper
#[repr(C)]
pub struct StringResult {
    ptr: *mut c_char,
    len: usize,
    error: HailMaryError,
}

// Improved hello_world with error handling
#[no_mangle]
pub extern "C" fn hello_world_safe() -> StringResult {
    match CString::new("hello, world") {
        Ok(message) => {
            let len = message.as_bytes().len();
            StringResult {
                ptr: message.into_raw(),
                len,
                error: HailMaryError::Success,
            }
        }
        Err(_) => StringResult {
            ptr: std::ptr::null_mut(),
            len: 0,
            error: HailMaryError::AllocationFailed,
        }
    }
}

// Safe string deallocation with error reporting
#[no_mangle]
pub extern "C" fn free_string_safe(s: *mut c_char) -> HailMaryError {
    if s.is_null() {
        return HailMaryError::NullPointer;
    }
    
    unsafe {
        let _ = CString::from_raw(s);
    }
    
    HailMaryError::Success
}

// Memory allocation for Wazero
#[no_mangle]
pub extern "C" fn hail_mary_alloc(size: usize) -> *mut u8 {
    if size == 0 {
        return std::ptr::null_mut();
    }
    
    let mut buffer = Vec::with_capacity(size);
    let ptr = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn hail_mary_free(ptr: *mut u8, size: usize) {
    if ptr.is_null() || size == 0 {
        return;
    }
    
    let _ = Vec::from_raw_parts(ptr, 0, size);
}
```

## Go統合（Wazeroホスト）

### 型安全なGoラッパー

```go
package wasm

import (
    "context"
    "errors"
    "unsafe"
    
    "github.com/tetratelabs/wazero"
)

// HailMaryError matches Rust enum
type HailMaryError uint32

const (
    Success HailMaryError = iota
    NullPointer
    InvalidUtf8
    AllocationFailed
)

// StringResult matches Rust struct
type StringResult struct {
    Ptr   uint32
    Len   uint32
    Error HailMaryError
}

type WasmModule struct {
    runtime  wazero.Runtime
    module   wazero.CompiledModule
    instance wazero.Instance
}

func (w *WasmModule) HelloWorld(ctx context.Context) (string, error) {
    // Call the safe version
    results, err := w.instance.ExportedFunction("hello_world_safe").Call(ctx)
    if err != nil {
        return "", err
    }
    
    // Parse result structure (assuming 64-bit return)
    ptr := uint32(results[0] & 0xFFFFFFFF)
    len := uint32(results[0] >> 32)
    errCode := HailMaryError(results[1])
    
    if errCode != Success {
        return "", errors.New("WASM error: " + errCode.String())
    }
    
    // Read string from memory
    memory := w.instance.Memory()
    bytes, err := memory.Read(ptr, len)
    if err != nil {
        return "", err
    }
    
    // Free the string
    freeResult, err := w.instance.ExportedFunction("free_string_safe").Call(ctx, uint64(ptr))
    if err != nil {
        return "", err
    }
    
    if HailMaryError(freeResult[0]) != Success {
        return "", errors.New("failed to free WASM memory")
    }
    
    return string(bytes), nil
}
```

## ベストプラクティスまとめ

### やるべきこと ✅
1. **エラーコードを使用**: enumによる明示的なエラーハンドリング
2. **すべてのポインタを検証**: 参照前のNULLチェック
3. **allocation/deallocationペアを提供**: メモリリーク防止
4. **`#[repr(C)]`を使用**: 一貫したメモリレイアウトを保証
5. **所有権を文書化**: 明確なメモリ所有権の移譲
6. **RAIIパターンを使用**: 可能な限り自動リソース管理
7. **FFI表面を最小限に**: 攻撃対象領域の削減

### やってはいけないこと ❌
1. **safer-ffiを使用しない**: WebAssemblyと非互換
2. **wasm-bindgenを使用しない**: Wazeroと非互換
3. **メモリレイアウトを仮定しない**: 常に`#[repr(C)]`を使用
4. **エラーを無視しない**: 常にallocation失敗を処理
5. **アロケータを混在させない**: 一貫したallocation戦略を使用
6. **外部入力を信頼しない**: 常に検証

## パフォーマンスの考慮事項

### メモリオーバーヘッド
- 初期WASM線形メモリ: 17ページ（1.1MB）
- 各allocationにはVec容量追跡のオーバーヘッドあり
- 頻繁なallocationにはプーリングを検討

### 最適化フラグ
```toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link Time Optimization
strip = true        # Strip symbols
```

### 代替アロケータ
より小さなバイナリのために`wee_alloc`を検討：
```toml
[dependencies]
wee_alloc = "0.4"
```

```rust
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
```

## テスト戦略

### ユニットテスト（Rust）
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_safe_allocation() {
        let ptr = hail_mary_alloc(100);
        assert!(!ptr.is_null());
        unsafe { hail_mary_free(ptr, 100); }
    }
    
    #[test]
    fn test_error_handling() {
        let result = hello_world_safe();
        assert_eq!(result.error, HailMaryError::Success);
        assert!(!result.ptr.is_null());
        unsafe { free_string_safe(result.ptr); }
    }
}
```

### 統合テスト（Go）
```go
func TestWasmIntegration(t *testing.T) {
    module := LoadWasmModule(t)
    defer module.Close()
    
    result, err := module.HelloWorld(context.Background())
    require.NoError(t, err)
    assert.Equal(t, "hello, world", result)
}
```

## 移行パス

### フェーズ1: エラーハンドリングの追加
- エラーコードenumの実装
- 結果ラッパー構造体の追加
- 既存関数と並行して安全バージョンを作成

### フェーズ2: メモリ管理の改善
- allocation/deallocationペアの実装
- 複雑な型用のRAIIラッパーの追加
- メモリ所有権の文書化

### フェーズ3: 型安全性の強化
- 複雑な型用の不透明ハンドルの使用
- バリデーション層の実装
- 包括的なテストの追加

### フェーズ4: ドキュメント化
- すべてのFFI関数の文書化
- 使用例の作成
- 統合ガイドの追加

## 結論

safer-ffiや類似のライブラリはWebAssemblyで使用できませんが、以下の方法で型安全性を実現できます：

1. 型安全パターンの**手動実装**
2. 結果型による**明示的なエラーハンドリング**
3. FFI境界での**バリデーション層**
4. リソース管理のための**RAIIパターン**
5. ユニットおよび統合レベルでの**包括的なテスト**

現在のHail Mary実装はWebAssemblyのベストプラクティスに従っています。推奨される改善は、非互換な依存関係を導入することなく、エラーハンドリングとバリデーションの追加に焦点を当てています。

## 参考文献

- [Wazero Rust Language Guide](https://wazero.io/languages/rust/)
- [WebAssembly Specification](https://webassembly.github.io/spec/core/)
- [Rust FFI Omnibus](http://jakegoulding.com/rust-ffi-omnibus/)
- [The Rustonomicon - FFI](https://doc.rust-lang.org/nomicon/ffi.html)
- [safer-ffi Documentation](https://getditto.github.io/safer_ffi/)
- [WebAssembly Linear Memory](https://radu-matei.com/blog/practical-guide-to-wasm-memory/)

## 付録: 調査タイムライン

- **初期依頼**: WebAssembly型安全性のためのsafer-ffi調査
- **リサーチフェーズ**: safer-ffi、cxx、wasm-bindgenの互換性分析
- **Wazero分析**: Wazero固有の制約とパターンの研究
- **パターン開発**: 型安全実装パターンの特定
- **文書化**: 調査結果と推奨事項のまとめ

---

*Hail MaryプロジェクトWebAssembly FFI調査の一環として作成*
*日付: 2025年8月11日*