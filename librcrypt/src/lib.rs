use rand::{CryptoRng, Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

const MAGIC: &[u8] = b"RCrypt";

// fn main() {
//     let key = "very secret secret key";
//     let message = "we ride at dawn";
//
//     let encoded = encode_base64(key, message);
//     let decoded = decode_from_base64(key, &encoded);
//     println!("{decoded}");
// }

// encrypt and encode in base64, with the magic byte and offset prepended
pub fn encrypt_base64(key: &str, offset: u128, message: &str) -> String {
    let encrypted_offset = encrypt_raw(key, 0, &offset.to_be_bytes());
    let encrypted_text = encrypt_raw(key, offset, message.as_bytes());
    let mut all = MAGIC.to_vec();
    all.extend_from_slice(&encrypted_offset);
    all.extend_from_slice(&encrypted_text);
    base64::encode(&all)
}

// encrypt with no encoding. does not include any metadata.
pub fn encrypt_raw(key: &str, offset: u128, data: &[u8]) -> Vec<u8> {
    let key_hash = blake3::hash(key.as_bytes());
    let mut rng = ChaCha20Rng::from_seed(key_hash.into());
    rng.set_word_pos(offset);
    let out_bytes: Vec<u8> = data
        .iter()
        .flat_map(|&in_byte| {
            let dup = in_byte as u16 | ((!in_byte as u16) << 8);
            let shuffled = shuffle_bits(&mut rng, dup);
            [
                ((shuffled & (255u16 << 8)) >> 8) as u8,
                (shuffled & 255u16) as u8,
            ]
        })
        .collect();
    out_bytes
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum DecryptError {
    #[error("Length is not a multiple of 2 (something is missing)")]
    LengthMismatch,
    #[error("Length is too short")]
    TooShort,
    #[error("Magic bytes are missing or damaged")]
    NoMagic,
    #[error("Invalid base-64 data (invalid charecters in input) {0:?}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("Message contains invalid utf-8")]
    InvalidUTF8(#[from] std::string::FromUtf8Error),
}

/// erorrs in decryption are NOT caused by key mismatch (except InvalidUTF8)
pub fn decrypt_base64(key: &str, message: &str) -> Result<String, DecryptError> {
    const OFFSET_LEN: usize = 16*2/* u128 byte len * 2 (encoding size increase) */;
    let bytes = base64::decode(message)?;
    if bytes.len() < MAGIC.len() + OFFSET_LEN {
        return Err(DecryptError::TooShort);
    }
    if &bytes[0..MAGIC.len()] != MAGIC {
        return Err(DecryptError::NoMagic);
    }
    let offset_data = &bytes[MAGIC.len()..][..OFFSET_LEN];
    let offset = u128::from_be_bytes(decrypt_raw(key, 0, offset_data)?.try_into().unwrap());

    let message_data = &bytes[MAGIC.len() + OFFSET_LEN..];
    let decrypted_message = decrypt_raw(key, offset, message_data)?;

    Ok(String::from_utf8(decrypted_message)?)
}

// offset is infered from encoded text (prefixed)
pub fn decrypt_raw(key: &str, offset: u128, data: &[u8]) -> Result<Vec<u8>, DecryptError> {
    let key_hash = blake3::hash(key.as_bytes());
    let mut rng = ChaCha20Rng::from_seed(key_hash.into());
    rng.set_word_pos(offset);
    if data.len() % 2 != 0 {
        return Err(DecryptError::LengthMismatch);
    }
    Ok(data
        .chunks(2)
        .map(|in_byte| {
            let in_byte = ((in_byte[0] as u16) << 8) | in_byte[1] as u16;
            let unshuffled = unshuffle_bits(&mut rng, in_byte);
            let [left, right] = [
                ((unshuffled & (255u16 << 8)) >> 8) as u8,
                (unshuffled & 255u16) as u8,
            ];
            assert_eq!(!left, right);
            right
        })
        .collect())
}

fn random_indicies<T: Rng + CryptoRng>(rng: &mut T) -> Vec<usize> {
    let mut avail = (0..16).collect::<Vec<_>>();
    let mut res = vec![];
    for _ in 0..16 {
        let selected = avail.remove(rng.sample(rand::distributions::Uniform::new(0, avail.len())));
        res.push(selected);
    }
    res
}

/// randomness consumed by this shoudl be consistant?
fn shuffle_bits<T: Rng + CryptoRng>(rng: &mut T, bytes: u16) -> u16 {
    // let random_idx = || rng.sample(rand::distributions::Uniform::new(0, 16));
    let mut res = 0;
    for (i, selected) in (0..16).zip(random_indicies(rng).into_iter()) {
        let cbit = (bytes & (1 << i)) >> i;
        res |= cbit << selected;
    }
    res
}

fn unshuffle_bits<T: Rng + CryptoRng>(rng: &mut T, bytes: u16) -> u16 {
    let mut res = 0;
    for (i, selected) in (0..16).zip(random_indicies(rng).into_iter()) {
        let sbit = (bytes & (1 << selected)) >> selected;
        res |= sbit << i;
    }
    res
}

#[test]
fn fuzz_shuffle() {
    let rng = ChaCha20Rng::from_seed(*blake3::hash("test key alskdjflk".as_bytes()).as_bytes());
    for i in 0..=u16::MAX {
        let mut r1 = rng.clone();
        let mut r2 = rng.clone();
        assert_eq!(i, unshuffle_bits(&mut r2, shuffle_bits(&mut r1, i)))
    }
}
