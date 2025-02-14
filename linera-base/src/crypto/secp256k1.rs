// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Defines secp256k1 signature primitives used by the Linera protocol.

use std::{fmt, str::FromStr};

use once_cell::sync::Lazy;
use secp256k1::{self, All, Message, Secp256k1};
use serde::{Deserialize, Serialize};

use super::{BcsHashable, BcsSignable, CryptoError, CryptoHash, HasTypeName};
use crate::doc_scalar;

/// Static Secp256k1 context for reuse.
pub static SECP256K1: Lazy<Secp256k1<All>> = Lazy::new(secp256k1::Secp256k1::new);

/// A secp256k1 secret key.
pub struct Secp256k1SecretKey(pub secp256k1::SecretKey);

/// A secp256k1 public key.
#[derive(Eq, PartialEq, Copy, Clone, PartialOrd, Ord, Hash)]
pub struct Secp256k1PublicKey(pub secp256k1::PublicKey);

/// Secp256k1 public/private key pair.
#[derive(Debug, PartialEq, Eq)]
pub struct Secp256k1KeyPair {
    /// Secret key.
    pub secret_key: Secp256k1SecretKey,
    /// Public key.
    pub public_key: Secp256k1PublicKey,
}

/// A Secp256k1 signature.
#[derive(Eq, PartialEq, Copy, Clone)]
pub struct Secp256k1Signature(pub secp256k1::ecdsa::Signature);

impl PartialEq for Secp256k1SecretKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl fmt::Debug for Secp256k1SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<redacted for Secp256k1 secret key>")
    }
}

impl Eq for Secp256k1SecretKey {}

impl Serialize for Secp256k1PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&hex::encode(self.0.serialize()))
        } else {
            serializer.serialize_newtype_struct("Secp256k1PublicKey", &self.0)
        }
    }
}

impl<'de> Deserialize<'de> for Secp256k1PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            let value = hex::decode(s).map_err(serde::de::Error::custom)?;
            let pk = secp256k1::PublicKey::from_slice(&value).map_err(serde::de::Error::custom)?;
            Ok(Secp256k1PublicKey(pk))
        } else {
            #[derive(Deserialize)]
            #[serde(rename = "Secp256k1PublicKey")]
            struct Foo(secp256k1::PublicKey);

            let value = Foo::deserialize(deserializer)?;
            Ok(Self(value.0))
        }
    }
}

impl FromStr for Secp256k1PublicKey {
    type Err = CryptoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pk = secp256k1::PublicKey::from_str(s)
            .map_err(|_| CryptoError::IncorrectPublicKeySize(0))?;
        Ok(Secp256k1PublicKey(pk))
    }
}

impl From<[u8; 33]> for Secp256k1PublicKey {
    fn from(value: [u8; 33]) -> Self {
        let pk = secp256k1::PublicKey::from_slice(&value).expect("Invalid public key");
        Secp256k1PublicKey(pk)
    }
}

impl TryFrom<&[u8]> for Secp256k1PublicKey {
    type Error = CryptoError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let pk = secp256k1::PublicKey::from_slice(value)
            .map_err(|_| CryptoError::IncorrectPublicKeySize(value.len()))?;
        Ok(Secp256k1PublicKey(pk))
    }
}

impl fmt::Display for Secp256k1PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = hex::encode(self.0.serialize());
        write!(f, "{}", s)
    }
}

impl fmt::Debug for Secp256k1PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0.serialize()[0..9]))
    }
}

impl<'de> BcsHashable<'de> for Secp256k1PublicKey {}

impl Secp256k1KeyPair {
    #[cfg(all(with_getrandom, with_testing))]
    /// Generates a new key-pair.
    pub fn generate() -> Self {
        let mut rng = rand::rngs::OsRng;
        Self::generate_from(&mut rng)
    }

    #[cfg(with_getrandom)]
    /// Generates a new key-pair from the given RNG. Use with care.
    pub fn generate_from<R: super::CryptoRng>(rng: &mut R) -> Self {
        let (sk, pk) = SECP256K1.generate_keypair(rng);
        Secp256k1KeyPair {
            secret_key: Secp256k1SecretKey(sk),
            public_key: Secp256k1PublicKey(pk),
        }
    }

    /// Returns public key for the key-pair.
    pub fn public(&self) -> &Secp256k1PublicKey {
        &self.public_key
    }
}

impl Secp256k1SecretKey {
    /// Returns a public key for the given secret key.
    pub fn public(&self) -> Secp256k1PublicKey {
        Secp256k1PublicKey(self.0.public_key(&SECP256K1))
    }

    /// Copies the key-pair, **including the secret key**.
    ///
    /// The `Clone` and `Copy` traits are deliberately not implemented for `KeyPair` to prevent
    /// accidental copies of secret keys.
    pub fn copy(&self) -> Secp256k1SecretKey {
        Secp256k1SecretKey(self.0.clone())
    }
}

impl Secp256k1PublicKey {
    /// Returns a public key for the given secret key.
    #[allow(dead_code)]
    fn from_secret_key(secret: &Secp256k1SecretKey) -> Self {
        secret.public()
    }

    /// A fake public key used for testing.
    #[cfg(with_testing)]
    pub fn test_key(name: u8) -> Secp256k1PublicKey {
        let addr = [name; secp256k1::constants::PUBLIC_KEY_SIZE];
        Secp256k1PublicKey(secp256k1::PublicKey::from_slice(&addr).unwrap())
    }
}

impl Secp256k1Signature {
    /// Computes a secp256k1 signature for [`value`] using the given [`secret`].
    /// It first serializes the `T` type and then creates the `CryptoHash` from the serialized bytes.
    pub fn new<'de, T>(value: &T, secret: &Secp256k1SecretKey) -> Self
    where
        T: BcsSignable<'de>,
    {
        let secp = secp256k1::Secp256k1::signing_only();
        let message = Message::from_digest(CryptoHash::new(value).as_bytes().0);
        let signature = secp.sign_ecdsa(&message, &secret.0);
        Secp256k1Signature(signature)
    }

    /// Verifies a batch of signatures.
    pub fn verify_batch<'a, 'de, T, I>(value: &'a T, votes: I) -> Result<(), CryptoError>
    where
        T: BcsSignable<'de>,
        I: IntoIterator<Item = (&'a Secp256k1PublicKey, &'a Secp256k1Signature)>,
    {
        let message = Message::from_digest(CryptoHash::new(value).as_bytes().0);
        for (author, signature) in votes {
            SECP256K1
                .verify_ecdsa(&message, &signature.0, &author.0)
                .map_err(|error| CryptoError::InvalidSignature {
                    error: error.to_string(),
                    type_name: T::type_name().to_string(),
                })?;
        }
        Ok(())
    }

    /// Checks a signature.
    pub fn check<'de, T>(&self, value: &T, author: &Secp256k1PublicKey) -> Result<(), CryptoError>
    where
        T: BcsSignable<'de> + fmt::Debug,
    {
        let message = Message::from_digest(CryptoHash::new(value).as_bytes().0);
        SECP256K1
            .verify_ecdsa(&message, &self.0, &author.0)
            .map_err(|error| CryptoError::InvalidSignature {
                error: error.to_string(),
                type_name: T::type_name().to_string(),
            })
    }
}

impl Serialize for Secp256k1Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&hex::encode(self.0.serialize_der()))
        } else {
            serializer.serialize_newtype_struct("Signature", &self.0)
        }
    }
}

impl<'de> Deserialize<'de> for Secp256k1Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            let value = hex::decode(s).map_err(serde::de::Error::custom)?;
            let sig =
                secp256k1::ecdsa::Signature::from_der(&value).map_err(serde::de::Error::custom)?;
            Ok(Secp256k1Signature(sig))
        } else {
            #[derive(Deserialize)]
            #[serde(rename = "Signature")]
            struct Foo(secp256k1::ecdsa::Signature);

            let value = Foo::deserialize(deserializer)?;
            Ok(Self(value.0))
        }
    }
}

impl fmt::Display for Secp256k1Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = hex::encode(self.0.serialize_der());
        write!(f, "{}", s)
    }
}

impl fmt::Debug for Secp256k1Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0.serialize_der()[0..8]))
    }
}

doc_scalar!(Secp256k1Signature, "A Secp256k1 signature value");

#[cfg(with_testing)]
mod secp256k1_tests {
    #[test]
    fn test_signatures() {
        use serde::{Deserialize, Serialize};

        use crate::crypto::{
            secp256k1::{Secp256k1KeyPair, Secp256k1Signature},
            BcsSignable, TestString,
        };

        #[derive(Debug, Serialize, Deserialize)]
        struct Foo(String);

        impl<'de> BcsSignable<'de> for Foo {}

        let keypair1 = Secp256k1KeyPair::generate();
        let keypair2 = Secp256k1KeyPair::generate();

        let ts = TestString("hello".into());
        let tsx = TestString("hellox".into());
        let foo = Foo("hello".into());

        let s = Secp256k1Signature::new(&ts, &keypair1.secret_key);
        assert!(s.check(&ts, &keypair1.public_key).is_ok());
        assert!(s.check(&ts, &keypair2.public_key).is_err());
        assert!(s.check(&tsx, &keypair1.public_key).is_err());
        assert!(s.check(&foo, &keypair1.public_key).is_err());
    }
}
