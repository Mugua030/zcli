use crate::{process_genpwd, TextSignFormat};
use anyhow::{Context, Result};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use std::{collections::HashMap, io::Read};

//use rand::rngs::OsRng;

// Sign for text
pub trait TextSigner {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

pub trait TextVerifier {
    fn verify(&self, reader: &mut dyn Read, sig: &[u8]) -> Result<bool>;
}

// encrypt
pub trait Encryptor {
    fn encrypt(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
    fn decrypt(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

pub struct EncryXChaCha20Poly1305 {
    cipher: ChaCha20Poly1305,
}

pub struct Blake3 {
    key: [u8; 32],
}

pub struct Ed25519Signer {
    key: SigningKey,
}

pub struct Ed25519Verifier {
    key: VerifyingKey,
}

impl Ed25519Verifier {
    pub fn try_new(key: impl AsRef<[u8]>) -> Result<Self> {
        let key = key.as_ref();
        let key = (&key[..32]).try_into()?;
        let key = VerifyingKey::from_bytes(key)?;
        Ok(Self { key })
    }
}

impl TextSigner for Blake3 {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        //let ret = blake3
        let ret = blake3::keyed_hash(&self.key, &buf);
        Ok(ret.as_bytes().to_vec())
    }
}

impl TextVerifier for Blake3 {
    fn verify(&self, reader: &mut dyn Read, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let ret = blake3::keyed_hash(&self.key, &buf);
        Ok(ret.as_bytes() == sig)
    }
}

impl TextSigner for Ed25519Signer {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let signature = self.key.sign(&buf);
        Ok(signature.to_bytes().to_vec())
    }
}

impl TextVerifier for Ed25519Verifier {
    fn verify(&self, reader: &mut dyn Read, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let sig = (&sig[..64]).try_into()?;
        let signature = Signature::from_bytes(sig);
        Ok(self.key.verify(&buf, &signature).is_ok())
    }
}

impl EncryXChaCha20Poly1305 {
    pub fn try_new() -> anyhow::Result<Self> {
        let key = ChaCha20Poly1305::generate_key(&mut OsRng);
        let cipher = ChaCha20Poly1305::new(&key);
        Ok(Self { cipher })
    }
}

impl Encryptor for EncryXChaCha20Poly1305 {
    fn encrypt(&self, reader: &mut dyn Read) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader
            .read_to_end(&mut buf)
            .with_context(|| "fail to read data")?;

        //nonce
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let en_data = match self.cipher.encrypt(&nonce, buf.as_slice()) {
            Ok(data) => data,
            Err(err) => return Err(anyhow::Error::msg(format!("CryptoError: {}", err))),
        };

        //println!("encrypt-buf: {:?}", String::from_utf8_lossy(&en_data));
        //let nonce_s = nonce.into
        Ok(en_data)
    }

    fn decrypt(&self, reader: &mut dyn Read) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader
            .read_to_end(&mut buf)
            .with_context(|| "failed to read data")?;

        let ciphertext = buf.as_slice();
        let nonce = Nonce::from_slice(&ciphertext[..12]);
        let dec_data = match self.cipher.decrypt(nonce, ciphertext) {
            Ok(data) => data,
            Err(err) => return Err(anyhow::Error::msg(format!("descrtoError: {}", err))),
        };

        Ok(dec_data)
    }
}

impl Blake3 {
    pub fn try_new(key: impl AsRef<[u8]>) -> Result<Self> {
        let key = key.as_ref();
        let key = (&key[..32]).try_into()?;
        Ok(Self::new(key))
    }

    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    fn generate() -> Result<HashMap<&'static str, Vec<u8>>> {
        let key = process_genpwd(32, true, true, true, true)?;
        let mut map = HashMap::new();
        //map.insert("blake3.txt", key.as_bytes().to_vec());
        map.insert("blake3.txt", key.as_bytes().to_vec());
        Ok(map)
    }
}

impl Ed25519Signer {
    pub fn try_new(key: impl AsRef<[u8]>) -> Result<Self> {
        let key = key.as_ref();
        let key = (&key[..32]).try_into()?;
        Ok(Self::new(key))
    }

    pub fn new(key: &[u8; 32]) -> Self {
        let key = SigningKey::from_bytes(key);
        Self { key }
    }

    pub fn generate() -> Result<HashMap<&'static str, Vec<u8>>> {
        let mut csprng = OsRng;
        let sk: SigningKey = SigningKey::generate(&mut csprng);
        let pk: VerifyingKey = (&sk).into();
        let mut map = HashMap::new();
        map.insert("ed25519.sk", sk.to_bytes().to_vec());
        map.insert("ed25519.pk", pk.to_bytes().to_vec());

        Ok(map)
    }
}

pub fn process_text_sign(
    reader: &mut dyn Read,
    key: &[u8],
    format: TextSignFormat,
) -> Result<Vec<u8>> {
    let signer: Box<dyn TextSigner> = match format {
        TextSignFormat::Blake3 => Box::new(Blake3::try_new(key)?),
        TextSignFormat::Ed25519 => Box::new(Ed25519Signer::try_new(key)?),
    };
    signer.sign(reader)
}

pub fn process_text_verify(
    reader: &mut dyn Read,
    key: &[u8],
    sig: &[u8],
    format: TextSignFormat,
) -> Result<bool> {
    let verifier: Box<dyn TextVerifier> = match format {
        TextSignFormat::Blake3 => Box::new(Blake3::try_new(key)?),
        TextSignFormat::Ed25519 => Box::new(Ed25519Verifier::try_new(key)?),
    };

    verifier.verify(reader, sig)
}

pub fn process_text_key_generate(format: TextSignFormat) -> Result<HashMap<&'static str, Vec<u8>>> {
    match format {
        TextSignFormat::Blake3 => Blake3::generate(),
        TextSignFormat::Ed25519 => Ed25519Signer::generate(),
    }
}

pub fn process_text_encypt(reader: &mut dyn Read) -> Result<Vec<u8>> {
    let xcha = EncryXChaCha20Poly1305::try_new()?;
    xcha.encrypt(reader)
}

pub fn process_text_decypt(reader: &mut dyn Read) -> Result<Vec<u8>> {
    let xcha = EncryXChaCha20Poly1305::try_new()?;
    xcha.decrypt(reader)
}
#[cfg(test)]
mod tests {
    use super::*;
    //use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    //const KEY: &[u8] = include_bytes!("../../fixtures/tmp.txt"); // open when test
    const KEY: &[u8] = &[1]; // error

    #[test]
    fn test_encrypt_decrypt() -> Result<()> {
        let cipher = EncryXChaCha20Poly1305::try_new()?;

        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let en_data = match cipher.cipher.encrypt(&nonce, KEY) {
            Ok(data) => data,
            Err(err) => return Err(anyhow::Error::msg(format!("CryptoError: {}", err))),
        };
        //println!("origin-content: {}", String::from_utf8_lossy(&KEY));
        //println!("encrypt-result: {}", String::from_utf8_lossy(&en_data));

        //let nonce_dec = Nonce::from_slice(&en_data.as_slice()[..12]);
        let ret = match cipher.cipher.decrypt(&nonce, en_data.as_slice()) {
            Ok(edata) => edata,
            Err(err) => return Err(anyhow::Error::msg(format!("DecryptError: {}", err))),
        };

        println!("descrypt-result: {}", String::from_utf8_lossy(&ret));

        Ok(())
    }

    /*
    const KEY: &[u8] = include_bytes!("../../fixtures/blake3.txt");

    #[test]
    fn test_process_text_sign() -> Result<()> {
        let mut reader = "hello".as_bytes();
        let mut reader2 = "hello".as_bytes();
        let format = TextSignFormat::Blake3;
        let sig = process_text_sign(&mut reader, KEY, format)?;
        let ret = process_text_verify(&mut reader2, KEY, &sig, format)?;

        assert!(ret);

        Ok(())
    }
    */
}
