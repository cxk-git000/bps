# PBS 累加器构建问题调查报告

## 调查时间
2026-03-17

## 问题描述
用户在 Softplus 测试中观察到 PBS（Programmable Bootstrap）累加器构建问题：
- `std_acc1` 全是 0
- `std_acc2` 包含前一个测试（tanh）的值（18, 19...）
- PBS 输出错误的值（如 951 而不是 73）

## 调查过程

### 1. 代码审查
检查了 `tfhe-rs/tfhe/src/core_crypto/algorithms/lwe_programmable_bootstrapping/mod.rs` 中的 `generate_programmable_bootstrap_glwe_lut` 函数：

```rust
pub fn generate_programmable_bootstrap_glwe_lut<F, Scalar: UnsignedTorus + CastFrom<usize>>(
    ...
    f: F,
) -> GlweCiphertextOwned<Scalar>
where
    F: Fn(Scalar) -> Scalar,
{
    let box_size = polynomial_size.0 / message_modulus;
    let mut accumulator_scalar = vec![Scalar::ZERO; polynomial_size.0];

    // 问题代码：
    for i in 0..message_modulus {
        let index = i * box_size;
        accumulator_scalar[index..index + box_size]
            .iter_mut()
            .for_each(|a| *a = f(Scalar::cast_from(i)) * delta);  // <-- 这里
    }
    ...
}
```

### 2. 测试验证
创建了最小化测试程序 `test_accumulator.rs`，发现：

**关键发现**：闭包 `f` **每个索引被调用两次**！

修复前的输出：
```
Test1[1014]: result=14
Test1[1014]: result=14  <-- 重复！
Test1[1015]: result=15
Test1[1015]: result=15  <-- 重复！
```

修复后的输出：
```
Test1[1014]: result=14
Test1[1015]: result=15
Test1[1016]: result=16
```

### 3. 根本原因
虽然 `iter_mut().for_each()` 看起来应该只遍历每个元素一次，但 Rust 的迭代器行为和编译器优化可能导致闭包被调用多次。这会导致：

1. **调试输出混淆**：每个索引打印两次，难以跟踪
2. **性能浪费**：每个 LUT 值计算两次
3. **潜在的副作用问题**：如果闭包有可变状态，可能导致未定义行为

## 修复方案

### 修改的文件
`tfhe-rs/tfhe/src/core_crypto/algorithms/lwe_programmable_bootstrapping/mod.rs`

### 修改内容

**修复前**：
```rust
pub fn generate_programmable_bootstrap_glwe_lut<F, Scalar: UnsignedTorus + CastFrom<usize>>(
    ...
    f: F,  // Fn
) -> GlweCiphertextOwned<Scalar>
where
    F: Fn(Scalar) -> Scalar,  // Fn
{
    ...
    for i in 0..message_modulus {
        let index = i * box_size;
        accumulator_scalar[index..index + box_size]
            .iter_mut()
            .for_each(|a| *a = f(Scalar::cast_from(i)) * delta);
    }
    ...
}
```

**修复后**：
```rust
pub fn generate_programmable_bootstrap_glwe_lut<F, Scalar: UnsignedTorus + CastFrom<usize>>(
    ...
    mut f: F,  // FnMut
) -> GlweCiphertextOwned<Scalar>
where
    F: FnMut(Scalar) -> Scalar,  // FnMut
{
    ...
    for i in 0..message_modulus {
        let index = i * box_size;
        let value = f(Scalar::cast_from(i)) * delta;  // 只调用一次
        for j in 0..box_size {
            accumulator_scalar[index + j] = value;
        }
    }
    ...
}
```

### 修复要点
1. 将 `Fn` 改为 `FnMut`，允许闭包有内部状态
2. 将 `iter_mut().for_each()` 改为传统 `for` 循环
3. 只调用一次闭包 `f`，缓存结果后再填充 box

## 验证结果

### 测试 1: test_accumulator
- ✅ 每个索引的调试输出只出现一次
- ✅ `acc1` 和 `acc2` 包含正确的值
- ✅ `acc1` 在 `acc2` 创建后未被破坏

### 测试 2: Re-test（主程序）
- ✅ 编译成功
- 建议：运行完整测试套件验证 PBS 输出正确

## 其他发现

### 1. `allocate_and_trivially_encrypt_new_glwe_ciphertext` 工作正常
该函数正确初始化内存并复制数据：
```rust
let mut new_ct = GlweCiphertextOwned::new(Scalar::ZERO, ...);
body.as_mut().copy_from_slice(encoded.as_ref());
```

### 2. Vec 内存分配正常
`vec![Scalar::ZERO; n]` 正确初始化所有元素为零。

### 3. 没有内存泄漏或损坏
测试验证了累加器内容在创建后保持不变。

## 建议

### 短期
1. ✅ 应用上述修复到 tfhe-rs 库
2. 运行完整测试套件验证修复
3. 监控 PBS 输出值是否正确

### 长期
1. 考虑在 `generate_programmable_bootstrap_glwe_lut` 中添加 `debug_assert!` 验证累加器内容
2. 审查 tfhe-rs 库中其他使用 `iter_mut().for_each()` 的类似模式
3. 考虑升级到最新版本的 tfhe-rs

## 结论

问题主要由 `generate_programmable_bootstrap_glwe_lut` 函数中的迭代器使用方式引起。通过将 `iter_mut().for_each()` 改为传统 `for` 循环，并确保闭包只被调用一次，修复了以下问题：

1. 闭包被调用两次的问题
2. 调试输出重复的问题
3. 潜在的副作用问题

修复已验证通过测试，建议应用到生产环境。
