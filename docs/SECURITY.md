# Security Controls & Threats Safeguards

## 1. Path Traversal & Extraction Safety
- `SecurityValidator::sanitize_path` rejects paths containing relative directory traversal components (`..`), absolute path roots (`/etc/passwd`), or drive letters (`C:\`).
- Extraction targets are strictly constrained inside the designated destination folder.

## 2. Archive & Decompression Bomb Protection
- Maximum recursion depth limit for nested archives: Default **32 levels**.
- Maximum total extracted bytes ceiling enforcement.
- Memory allocation caps per chunk payload.

## 3. Cryptography Standards
- Password Key Derivation: **Argon2id** (memory-hard password hashing).
- Authenticated Encryption: **ChaCha20-Poly1305** (AEAD cipher ensuring payload confidentiality and authentication).
- Salt Generation: 128-bit cryptographically secure random salt.
- Nonce Generation: 96-bit unique random nonce per archive.
