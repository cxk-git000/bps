# SDR-PBS Shared-input Dual-Recovery Programmable Bootstrapping 问题最终解决方案

## 核心发现：标准 PBS 为什么能工作？

### 标准 PBS 的冗余机制

**Box Size = 2**（2048 位置 / 1024 boxes = 每个 box 占 2 个位置）

```
累加器布局：
位置 0: box 0 的值
位置 1: box 0 的值（重复！）
位置 2: box 1 的值
位置 3: box 1 的值（重复！）
...
```

**关键：每个值存 2 次！**

当盲旋转有 ±1 误差时：
- 目标提取位置 2j
- 实际可能提取到 2j-1, 2j, 或 2j+1
- 但位置 2j-1, 2j, 2j+1 都存的是同一个值！
- 所以结果仍然正确！

### SDR-PBS Shared-input Dual-Recovery Programmable Bootstrapping 的问题

**Box Size = 2，但两个位置存不同函数**

```
累加器布局（您的方案）：
位置 0: F1(j=0) = 0
位置 1: F2(j=0) = 1024（完全不同！）
位置 2: F1(j=1) = 1
位置 3: F2(j=1) = 1025（完全不同！）
...
```

当盲旋转有 ±1 误差时：
- 目标提取位置 2j（F1）和 2j+1（F2）
- 实际可能提取到 2j-1（F2 of j-1）和 2j（F1 of j）
- **F1 和 F2 完全混淆！**

## 解决方案

### 方案 1：增加 Box Size 到 4（推荐）

**核心思想：每个函数值存 2 次，提供冗余**

```rust
累加器布局（Box Size = 4）：
位置 4j:   F1(j)  （第1次）
位置 4j+1: F1(j)  （第2次，冗余）
位置 4j+2: F2(j)  （第1次）
位置 4j+3: F2(j)  （第2次，冗余）
```

**代码实现：**

```rust
// 构建累加器
let qacc = generate_programmable_bootstrap_glwe_lut(
    poly_size, glwe_dim.to_glwe_size(), 2048, ct_mod, delta_q,
    |x: u64| {
        let j = (x / 4) as usize;  // 512 boxes
        if j < 512 {
            let x_j = pair.x_min + (pair.x_max - pair.x_min) * (j as f64 + 0.5) / 512.0;
            match x % 4 {
                0 | 1 => quantize(f1(x_j), pair.q1_min, pair.q1_max),  // F1，存2次
                2 | 3 => quantize(f2(x_j), pair.q2_min, pair.q2_max),  // F2，存2次
                _ => 0
            }
        } else { 0 }
    },
);

// 使用
let j = (norm * 512.0).round() as usize;  // 512 boxes
let rotation_index = (4 * j) as u64;       // 每个 box 4 个位置

let pt_q = Plaintext(rotation_index.wrapping_mul(delta_q));
// ... 加密、模数切换、盲旋转 ...

let (s0, s1) = blind_rotate_and_extract_two_coefficients(&msed, &mut acc_buf, &fbsk);
let p0 = decrypt_lwe_ciphertext(&big_sk, &s0);
let p1 = decrypt_lwe_ciphertext(&big_sk, &s1);
let r0 = (((p0.0 as i64 + d/2) / d).rem_euclid(2048)) as u64;
let r1 = (((p1.0 as i64 + d/2) / d).rem_euclid(2048)) as u64;

// 解码（注意：s0 和 s1 可能交换，需要检测）
let v1 = r0.min(r1);  // F1 值较小
let v2 = r0.max(r1);  // F2 值较大
```

### 方案 2：解码时检测并纠正

如果不改变累加器结构，可以在解码时动态检测：

```rust
// 偏移编码：F1 在 0..1023，F2 在 1024..2047
let v1_candidate = if r0 < 1024 { r0 } else { r1 };  // 哪个是 F1？
let v2_candidate = if r0 >= 1024 { r0 } else { r1 }; // 哪个是 F2？
```

但这种方法在噪声大时仍然容易出错。

## 权衡

| 方案 | Boxes | 精度 | 速度 | 建议 |
|-----|-------|-----|------|-----|
| 原始 SDR-PBS Shared-input Dual-Recovery Programmable Bootstrapping | 1024 | 低 | 快 | 不推荐 |
| Box Size = 4 | 512 | 中 | 中 | 推荐 |
| 标准 PBS x2 | 1024 | 高 | 慢 | 基准 |

## 最终建议

使用 **Box Size = 4** 的方案：
1. 每个函数值存 2 次，提供冗余
2. 盲旋转 ±1 误差不会混淆 F1 和 F2
3. 512 boxes 对大多数应用足够
4. 速度仍是标准 PBS 的 2 倍

这是 SDR-PBS Shared-input Dual-Recovery Programmable Bootstrapping 能可靠工作的唯一方式。
