---
title: "Into the Uncrackable Cryptosystem: A Foundational Overview of One-Time Pads"
date: "2025-10-22"
author: "Icon The Great"
description: "Understanding One-Time Pads (OTPs)"
category: "Cryptography"
---

*This article is part of an ongoing research series exploring foundational cryptographic systems, their mathematical underpinnings, and their modern relevance.*

## Abstract

This article explores the **One-Time Pad (OTP)** :- a cryptographic scheme proven to offer *perfect secrecy* when implemented correctly. We discuss its mathematical foundations, encryption and decryption processes, historical applications, and practical limitations. Through examples and comparisons, we examine why the OTP remains both a theoretical ideal and a practical challenge in modern cryptography.

## Introduction

In the early days of global espionage and wartime intelligence, secrecy was not merely a tool, it was survival. During World War II, secret agents and military commanders relied on cipher machines, codebooks, and ingenious encryption methods to conceal their messages from enemy interception. Among these methods stood one system so secure that even with today’s supercomputers, it remains impossible to break when used correctly — the **One-Time Pad (OTP)**.

First patented in 1919 by Gilbert Vernam and Joseph Mauborgne, the One-Time Pad introduced the concept of *perfect secrecy*: a message so well-encrypted that no amount of computation or analysis could reveal the original text without the exact key. It was famously used in diplomatic and military communications, including by the Soviet Union during the Cold War.

Despite its age, the OTP continues to intrigue modern cryptographers and mathematicians alike. Its simplicity hides a profound mathematical truth that security can be absolute, but only under uncompromising conditions. 

Before diving into how the One-Time Pad works, it’s essential to understand the fundamental concepts of **encryption** and **decryption**, which form the bedrock of all secure communication systems.

## Encryption and Decryption

TO understand OTPs, it is essential to establish a fundamental understanding of **encryption** and **decryption** algorithms. This section provides a concise overview of these core cryptographic processes, laying the groundwork for a deeper discussion on One Time Pads.

**Encryption** (denoted as `E`) is the process of transforming a plaintext message into an unreadable format called *ciphertext*, such that only authorized parties can recover the original information. The goal is to ensure confidentiality and integrity of communication in the presence of eavesdroppers.

A **key** `k` is used in conjunction with an encryption algorithm `E` to encode a message `m` into ciphertext `c`. Mathematically:

```E(k ∈ K, m ∈ M) = c```

To recover the original message, the recipient uses the corresponding **decryption** algorithm `D`, which is the inverse of `E`:

```D(k ∈ K, c ∈ C) = m```

In practice, once Alice encrypts a message using a shared secret key `k`, only Bob whom possesses the same key can decrypt it to obtain the original message. This *symmetric nature* of encryption ensures secure communication, provided the key remains secret.

## Properties of a Perfect One-Time Pad

A One-Time Pad achieves *perfect secrecy* when the following conditions are met:

- **Key Length Requirement:**  
  The encryption key `k ∈ K` must be at least as long as the plaintext message `m ∈ M`  

```k ≥ m```

- **Single-Use Key:**  
The key must be used only once and securely destroyed after use.

- **True Randomness:**  
The key must be generated using a source of true randomness, ensuring no predictable pattern can be exploited.

When these conditions are strictly followed, the OTP provides **information-theoretic security**, meaning that no amount of computational power can decrypt the ciphertext without knowledge of the key.

## Encrypting and Decrypting with a One-Time Pad

Let’s assume Alice wants to send a secret message `"baby"` to Bob. We will use the **American Standard Code for Information Interchange (ASCII)** to represent the characters numerically.

From the ASCII table:

<figure>
  <img src="https://upload.wikimedia.org/wikipedia/commons/thumb/2/26/ASCII_Table_%28suitable_for_printing%29.svg/1280px-ASCII_Table_%28suitable_for_printing%29.svg.png">
  <figcaption>Figure 1: ASCII Table (source: Wikipedia)</figcaption>
</figure>


| Character | Decimal | Binary   |
|------------|----------|----------|
| b          | 98       | 1100010  |
| a          | 97       | 1100001  |
| b          | 98       | 1100010  |
| y          | 121      | 1111001  |

Next, we generate a random key (the *pad*). Remember, in a perfect OTP, the key must be **random** and **of equal length** to the message.

Let’s assume our random key (in decimal) is `21 29 90 45`. Converting these to binary using ASCII:

| Key (Decimal) | ASCII Symbol | Binary   |
|----------------|---------------|----------|
| 21             | NAK           | 0010101  |
| 29             | GS            | 0011101  |
| 90             | [             | 1011010  |
| 45             | -             | 0101101  |

Now comes the **XOR (Exclusive OR)** operation `⊕`.

**Definition:**  
Exclusive OR (XOR), or *logical inequality*, is a binary operation that outputs `1` if and only if its two inputs differ.  

Example:

```A = 101```
```B = 111```

```A ⊕ B = 010```

### Encryption

Now that we understand XOR, let’s encrypt our message.

| Message | Binary   | Key | Binary (Key) | XOR Result | Decimal | Cipher |
|----------|-----------|-----|---------------|-------------|----------|---------|
| b        | 1100010   | NAK | 0010101       | 1110111     | 119      | W       |
| a        | 1100001   | GS  | 0011101       | 1111100     | 124      | I       |
| b        | 1100010   | [   | 1011010       | 0111000     | 56       | 8       |
| y        | 1111001   | -   | 0101101       | 1010100     | 84       | T       |

Thus, instead of Alice sending the plaintext `"baby"` to Bob, she sends the ciphertext **`WI8T`**.

### Decryption

To decrypt, Bob uses the same key and applies XOR again:

| Cipher | Binary   | Key | Binary (Key) | XOR Result | Plain |
|---------|-----------|-----|---------------|-------------|--------|
| W       | 1110111   | NAK | 0010101       | 1100010     | b      |
| I       | 1111100   | GS  | 0011101       | 1100001     | a      |
| 8       | 0111000   | [   | 1011010       | 1100010     | b      |
| T       | 1010100   | -   | 0101101       | 1111001     | y      |

Result: **"baby"** — the original plaintext message.

## Example Implementation in Rust

```impls.rs```
```rust
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};

// ---------- Core Logic ----------

pub fn to_ascii(input: String) -> Vec<u8> {
    input.chars().map(|c| c as u8).collect()
}

pub fn to_binary(input: Vec<u8>) -> Vec<String> {
    input.iter().map(|b| format!("{:08b}", b)).collect()
}

pub fn xoring(input: Vec<String>, key: Vec<String>) -> Vec<u8> {
    input
        .iter()
        .enumerate()
        .map(|(i, byte)| {
            let b1 = u8::from_str_radix(byte, 2).unwrap();
            let b2 = u8::from_str_radix(&key[i % key.len()], 2).unwrap();
            b1 ^ b2
        })
        .collect()
}

pub fn generate_random_key_of_same_length(ascii: Vec<u8>) -> Vec<u8> {
    let mut rng = rand::rng();
    (0..ascii.len())
        .map(|_| rng.random_range(0..=255))
        .collect()
}

pub fn encrypt(input: String, key: Vec<u8>) -> Vec<u8> {
    let ascii = to_ascii(input);
    let ascii_binary = to_binary(ascii.clone());
    let key_binary = to_binary(key.clone());
    xoring(ascii_binary, key_binary)
}

pub fn decrypt(cipher_bytes: Vec<u8>, key: Vec<u8>) -> String {
    let cipher_binary = to_binary(cipher_bytes);
    let key_binary = to_binary(key);
    let xored = xoring(cipher_binary, key_binary);
    xored.iter().map(|&b| b as char).collect()
}

// ---------- HEX HELPERS ----------
// Convert bytes to hex string and vice versa

pub fn to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<String>>()
        .join("")
}

pub fn from_hex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

// ---------- JSON STRUCTS ----------

#[derive(Serialize, Deserialize)]
pub struct CipherData {
    pub cipher_hex: String,
    pub key_hex: String,
    pub plaintext_hint: Option<String>, // optional field to remind what it was
}

// ---------- File Helpers ----------

pub fn save_to_json(filename: &str, data: &CipherData) -> io::Result<()> {
    let json = serde_json::to_string_pretty(data).unwrap();
    let mut file = fs::File::create(filename)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

pub fn load_from_json(filename: &str) -> io::Result<CipherData> {
    let contents = fs::read_to_string(filename)?;
    let data: CipherData = serde_json::from_str(&contents).unwrap();
    Ok(data)
}

```

```main.rs```
```rust
mod impls;
use impls::*;
use std::io;

fn main() {
    println!("Do you want to (E)ncrypt or (D)ecrypt?");
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    let choice = choice.trim().to_lowercase();

    if choice == "e" || choice == "encrypt" {
        println!("Enter plaintext:");
        let mut plaintext = String::new();
        io::stdin().read_line(&mut plaintext).unwrap();
        let plaintext = plaintext.trim().to_string();

        let ascii = to_ascii(plaintext.clone());
        let key = generate_random_key_of_same_length(ascii.clone());
        let cipher_bytes = encrypt(plaintext.clone(), key.clone());

        let cipher_hex = to_hex(&cipher_bytes);
        let key_hex = to_hex(&key);

        println!("\n✅ Encryption complete!");
        println!("Plaintext: {}", plaintext);
        println!("Cipher (hex): {}", cipher_hex);
        println!("Key (hex): {}", key_hex);

        // Save to file
        let data = CipherData {
            cipher_hex: cipher_hex.clone(),
            key_hex: key_hex.clone(),
            plaintext_hint: Some(plaintext.clone()),
        };

        println!("\nEnter filename to save (e.g. secret.json):");
        let mut filename = String::new();
        io::stdin().read_line(&mut filename).unwrap();
        let filename = filename.trim();

        if let Err(err) = save_to_json(filename, &data) {
            eprintln!("❌ Failed to save file: {}", err);
        } else {
            println!("💾 Saved to '{}'", filename);
        }
    } else if choice == "d" || choice == "decrypt" {
        println!("Enter filename of JSON to decrypt (e.g. secret.json):");
        let mut filename = String::new();
        io::stdin().read_line(&mut filename).unwrap();
        let filename = filename.trim();

        match load_from_json(filename) {
            Ok(data) => {
                let cipher_bytes = from_hex(&data.cipher_hex);
                let key_bytes = from_hex(&data.key_hex);
                let decrypted = decrypt(cipher_bytes, key_bytes);

                println!("\n✅ Decryption complete!");
                println!("Decrypted text: {}", decrypted);
            }
            Err(err) => eprintln!("❌ Failed to read file: {}", err),
        }
    } else {
        println!("Invalid choice — please enter E or D.");
    }
}
```

you can run and play with the cli app, here is the full code on [Github](https://github.com/IconTheGreat/otp)



## Limitations and Challenges

While the One-Time Pad is **mathematically proven** to be unbreakable under ideal conditions, it is **impractical** for widespread use. Some of its key challenges include:

- Generating a truly random key of the same length as the plaintext is difficult.  
- Both parties must securely exchange and manage identical keys.  
- Secure key storage requires significant resources.  
- Keys must be destroyed after first use, reusing them breaks security.  
- The sender’s identity cannot be cryptographically verified.

## Conclusion

The One-Time Pad remains a cornerstone of cryptographic theory, serving as the only known encryption method that guarantees *absolute secrecy*. Although impractical for most modern communication systems due to its stringent key requirements, the OTP continues to inspire contemporary cryptographic protocols and theoretical models of secure communication.

---
