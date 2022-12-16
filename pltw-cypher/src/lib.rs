use rand::{CryptoRng, Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

// fn main() {
//     let key = "very secret secret key";
//     let message = "we ride at dawn";
//
//     let encoded = encode_base64(key, message);
//     let decoded = decode_from_base64(key, &encoded);
//     println!("{decoded}");
// }

pub fn encode_base64(key: &str, message: &str) -> String {
    let key_hash = blake3::hash(key.as_bytes());
    let mut rng = ChaCha20Rng::from_seed(key_hash.into());
    let out_bytes: Vec<u8> = message
        .as_bytes()
        .iter()
        .map(|&in_byte| {
            let dup = in_byte as u16 | ((!in_byte as u16) << 8);
            let shuffled = shuffle_bits(&mut rng, dup);
            [
                ((shuffled & (255u16 << 8)) >> 8) as u8,
                (shuffled & 255u16) as u8,
            ]
        })
        .flatten()
        .collect();
    base64::encode(out_bytes)
}

pub fn decode_from_base64(key: &str, encoded: &str) -> String {
    let key_hash = blake3::hash(key.as_bytes());
    let mut rng = ChaCha20Rng::from_seed(key_hash.into());
    let decoded = base64::decode(encoded).unwrap();
    assert_eq!(decoded.len() % 2, 0);
    let out_bytes: Vec<u8> = decoded
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
        .collect();
    String::from_utf8(out_bytes).unwrap()
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
