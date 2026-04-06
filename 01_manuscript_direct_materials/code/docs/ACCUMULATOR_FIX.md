# PBS 累加器构建问题修复

## 问题描述
在 Softplus 测试中观察到：
- `std_acc1` 全是 0
- `std_acc2` 包含前一个测试（tanh）的值（18, 19...）

## 根本原因

### 1. 闭包被多次调用
`generate_programmable_bootstrap_glwe_lut` 函数中的迭代器代码：
```rust
accumulator_scalar[index..index + box_size]
    .iter_mut()
    .for_each(|a| *a = f(Scalar::cast_from(i)) * delta);
```

由于 Rust 的迭代器行为，闭包 `f` **每个索引被调用两次**。这导致：
- 调试输出重复
- 如果闭包有副作用（如打印），会看到重复的输出
- 性能浪费

### 2. 潜在的内存初始化问题
虽然代码使用了 `vec![Scalar::ZERO; n]` 来初始化内存，但在某些情况下：
- Vec 的内存分配器可能重用之前释放的内存
- 如果之前的内存没有被正确清零，可能看到旧数据

## 解决方案

### 方案 1：修复 tfhe-rs 库（推荐）

修改 `tfhe-rs/tfhe/src/core_crypto/algorithms/lwe_programmable_bootstrapping/mod.rs`：

```rust
pub fn generate_programmable_bootstrap_glwe_lut<F, Scalar: UnsignedTorus + CastFrom<usize>>(
    polynomial_size: PolynomialSize,
    glwe_size: GlweSize,
    message_modulus: usize,
    ciphertext_modulus: CiphertextModulus<Scalar>,
    delta: Scalar,
    mut f: F,  // 改为 FnMut
) -> GlweCiphertextOwned<Scalar>
where
    F: FnMut(Scalar) -> Scalar,  // 改为 FnMut
{
    let box_size = polynomial_size.0 / message_modulus;
    let mut accumulator_scalar = vec![Scalar::ZERO; polynomial_size.0];

    // 使用传统 for 循环替代 iter_mut().for_each()
    for i in 0..message_modulus {
        let index = i * box_size;
        let value = f(Scalar::cast_from(i)) * delta;  // 只调用一次闭包
        for j in 0..box_size {
            accumulator_scalar[index + j] = value;
        }
    }

    let half_box_size = box_size / 2;

    if ciphertext_modulus.is_compatible_with_native_modulus() {
        for a_i in accumulator_scalar[0..half_box_size].iter_mut() {
            *a_i = (*a_i).wrapping_neg();
        }
    } else {
        let modulus: Scalar = ciphertext_modulus.get_custom_modulus().cast_into();
        for a_i in accumulator_scalar[0..half_box_size].iter_mut() {
            *a_i = (*a_i).wrapping_neg_custom_mod(modulus);
        }
    }

    accumulator_scalar.rotate_left(half_box_size);

    let accumulator_plaintext = PlaintextList::from_container(accumulator_scalar);

    allocate_and_trivially_encrypt_new_glwe_ciphertext(
        glwe_size,
        &accumulator_plaintext,
        ciphertext_modulus,
    )
}
```

**改进点：**
1. 将 `Fn` 改为 `FnMut`，允许闭包有内部状态
2. 将 `iter_mut().for_each()` 改为传统 `for` 循环
3. 只调用一次闭包 `f`，缓存结果

### 方案 2：修改 main.rs（临时 Workaround）

如果不希望修改 tfhe-rs 库，可以在 main.rs 中：

1. **预先计算 LUT 值**：
```rust
// 在构建累加器之前预先计算所有值
let lut_values1: Vec<u64> = (0..1024)
    .map(|i| {
        let x = pair.test_x_min
            + (pair.test_x_max - pair.test_x_min)
            * ((i as f64 + 0.5) / 1024.0);
        (pair.quantize1)(x)
    })
    .collect();

let lut_values2: Vec<u64> = (0..1024)
    .map(|i| {
        let x = pair.test_x_min
            + (pair.test_x_max - pair.test_x_min)
            * ((i as f64 + 0.5) / 1024.0);
        (pair.quantize2)(x)
    })
    .collect();

// 使用预先计算的值构建累加器
let std_acc1 = generate_programmable_bootstrap_glwe_lut(
    polynomial_size,
    glwe_dimension.to_glwe_size(),
    1024,
    ciphertext_modulus,
    delta_std,
    |i: u64| lut_values1[i as usize],
);

let std_acc2 = generate_programmable_bootstrap_glwe_lut(
    polynomial_size,
    glwe_dimension.to_glwe_size(),
    1024,
    ciphertext_modulus,
    delta_std,
    |i: u64| lut_values2[i as usize],
);
```

2. **添加验证检查**：
```rust
// 构建累加器后验证内容
{
    let body1 = std_acc1.get_body();
    let nonzeros1 = body1.as_ref().iter().filter(|&&v| v != 0).count();
    if nonzeros1 == 0 {
        panic!("std_acc1 全是 0！");
    }
    println!("std_acc1 非零元素数量: {}", nonzeros1);
}
```

## 验证修复

运行测试后应该看到：
1. 每个索引的调试输出只出现一次（而不是两次）
2. `std_acc1` 和 `std_acc2` 包含正确的值，没有内存污染
3. PBS 输出正确的值（如 73 而不是 951）

## 附加建议

1. **使用 `RUSTFLAGS="-Z sanitizer=address"`** 运行测试，检查内存问题
2. **在 Windows 上**，可以尝试设置 `set RUST_MIN_STACK=16777216` 增加栈大小
3. **如果问题仍然存在**，考虑升级到最新版本的 tfhe-rs，可能已经修复了相关问题
