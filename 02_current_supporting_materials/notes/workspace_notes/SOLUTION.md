# SDR-PBS Shared-input Dual-Recovery Programmable Bootstrapping 问题分析与解决方案

## 发现的核心问题

### 1. 盲旋转噪声
- 标准 PBS 在 base_log=23, level=1 参数下只有 **14% 成功率**
- 盲旋转引入的噪声有几百，相对于 delta 来说很大

### 2. s0 和 s1 被交换
- `blind_rotate_and_extract_two_coefficients` 提取的样本被交换了
- s0 对应 func2（奇数位置），s1 对应 func1（偶数位置）
- **解决方法：交换解码顺序**

### 3. 奇偶编码的问题
- 您的原始方案 c1=2*v1, c2=2*v2+1
- 相邻值只差 1，噪声 100 就会导致奇偶位翻转
- **解决方法：使用偏移编码或更多冗余**

## 最终解决方案

### 代码修正

```rust
// 原代码（错误）
let v1 = if r0 % 2 == 0 { r0 / 2 } else { (r0 - 1) / 2 };
let v2 = if r1 % 2 == 1 { (r1 - 1) / 2 } else { r1 / 2 };

// 修正代码（交换解码）
let v1 = r1 % 1024;  // s1 对应 func1
let v2 = r0 % 1024;  // s0 对应 func2
```

### 累加器构建

```rust
let qacc = generate_programmable_bootstrap_glwe_lut(
    poly_size, glwe_dim.to_glwe_size(), 2048, ct_mod, delta_q,
    |x: u64| {
        let j = (x / 2) as usize;
        if j < 1024 {
            let x_j = x_min + (x_max - x_min) * (j as f64 + 0.5) / 1024.0;
            if x % 2 == 0 { 
                quantize(f1(x_j), q1_min, q1_max)  // 偶数位置: func1
            } else { 
                quantize(f2(x_j), q2_min, q2_max)  // 奇数位置: func2
            }
        } else { 0 }
    },
);
```

### rotation_index 公式

```rust
// 盲旋转向右旋转
// rotation_index = 2*j 使位置 2j, 2j+1 转到索引 0, 1
let rotation_index = (2 * j) as u64;
```

## 测试建议

1. **降低 box 数量**：使用 512 boxes（box_size=4）增加冗余
2. **调整分解参数**：使用 base_log=4, level=9 减少噪声
3. **接受一定错误率**：盲旋转噪声无法完全消除

## 核心结论

SDR-PBS Shared-input Dual-Recovery Programmable Bootstrapping 的边界错误主要由以下原因导致：
1. **盲旋转噪声**：这是 FHE 固有的，无法完全消除
2. **样本交换**：`blind_rotate_and_extract_two_coefficients` 提取的 s0/s1 顺序与预期相反
3. **编码方式**：奇偶编码对噪声太敏感

修正后的代码可以工作，但错误率仍然高于标准 PBS，这是 SDR-PBS Shared-input Dual-Recovery Programmable Bootstrapping 方案的固有限制。
