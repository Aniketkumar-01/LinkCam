use sha2::{Digest, Sha256};

/// Derives a 32-byte symmetric transport encryption key from the pairing token and session ID.
pub fn derive_session_key(token: &str, session_id: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}:{}:linkcam_secure_v1", token, session_id).as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

/// Constructs a deterministic 12-byte nonce from packet sequence and timestamp.
pub fn construct_nonce(sequence: u32, timestamp: u64) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    nonce[0..4].copy_from_slice(&sequence.to_le_bytes());
    nonce[4..12].copy_from_slice(&timestamp.to_le_bytes());
    nonce
}

/// ChaCha20 (RFC 8439) stream cipher quarter round operating on the 16-word state
#[inline(always)]
fn quarter_round(x: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
    x[a] = x[a].wrapping_add(x[b]);
    x[d] ^= x[a];
    x[d] = x[d].rotate_left(16);

    x[c] = x[c].wrapping_add(x[d]);
    x[b] ^= x[c];
    x[b] = x[b].rotate_left(12);

    x[a] = x[a].wrapping_add(x[b]);
    x[d] ^= x[a];
    x[d] = x[d].rotate_left(8);

    x[c] = x[c].wrapping_add(x[d]);
    x[b] ^= x[c];
    x[b] = x[b].rotate_left(7);
}

/// ChaCha20 block function generating 64 bytes of keystream
fn chacha20_block(key: &[u8; 32], nonce: &[u8; 12], counter: u32) -> [u8; 64] {
    let mut state = [0u32; 16];

    // Constants: "expand 32-byte k"
    state[0] = 0x61707865;
    state[1] = 0x3320646e;
    state[2] = 0x79622d32;
    state[3] = 0x6b206574;

    // Key (8 words)
    for i in 0..8 {
        state[4 + i] = u32::from_le_bytes([
            key[i * 4],
            key[i * 4 + 1],
            key[i * 4 + 2],
            key[i * 4 + 3],
        ]);
    }

    // Counter (1 word)
    state[12] = counter;

    // Nonce (3 words)
    for i in 0..3 {
        state[13 + i] = u32::from_le_bytes([
            nonce[i * 4],
            nonce[i * 4 + 1],
            nonce[i * 4 + 2],
            nonce[i * 4 + 3],
        ]);
    }

    let mut working_state = state;

    // 20 rounds (10 column rounds + 10 diagonal rounds)
    for _ in 0..10 {
        // Column round
        quarter_round(&mut working_state, 0, 4, 8, 12);
        quarter_round(&mut working_state, 1, 5, 9, 13);
        quarter_round(&mut working_state, 2, 6, 10, 14);
        quarter_round(&mut working_state, 3, 7, 11, 15);

        // Diagonal round
        quarter_round(&mut working_state, 0, 5, 10, 15);
        quarter_round(&mut working_state, 1, 6, 11, 12);
        quarter_round(&mut working_state, 2, 7, 8, 13);
        quarter_round(&mut working_state, 3, 4, 9, 14);
    }

    let mut output = [0u8; 64];
    for i in 0..16 {
        let val = working_state[i].wrapping_add(state[i]);
        output[i * 4..i * 4 + 4].copy_from_slice(&val.to_le_bytes());
    }

    output
}

/// In-place ChaCha20 encryption/decryption (stream cipher symmetry)
pub fn chacha20_crypt(data: &mut [u8], key: &[u8; 32], nonce: &[u8; 12], initial_counter: u32) {
    let mut counter = initial_counter;
    let chunks = data.chunks_mut(64);

    for chunk in chunks {
        let block = chacha20_block(key, nonce, counter);
        for (b, k) in chunk.iter_mut().zip(block.iter()) {
            *b ^= *k;
        }
        counter = counter.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chacha20_roundtrip() {
        let key = derive_session_key("test_token_12345", "test_session_id_67890");
        let nonce = construct_nonce(1, 1729450000000);
        let original_data = b"Hello LinkCam encrypted video frame payload!";

        let mut data = original_data.to_vec();
        chacha20_crypt(&mut data, &key, &nonce, 1);
        assert_ne!(&data[..], original_data);

        // Decrypt with same key, nonce, counter
        chacha20_crypt(&mut data, &key, &nonce, 1);
        assert_eq!(&data[..], original_data);
    }
}
