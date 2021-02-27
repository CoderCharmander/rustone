use openssl::{pkey::Private, rsa::Rsa};
use rand::*;
use rustone::errors::*;

pub fn generate_access_key() -> String {
    let key = thread_rng()
        .sample_iter(&distributions::Alphanumeric)
        .take(128)
        .map(char::from)
        .collect();
    key
}

pub fn generate_keypair() -> Result<Rsa<Private>> {
    let rsa = openssl::rsa::Rsa::generate(4096).chain_err(|| "failed to generate rsa keypair")?;
    Ok(rsa)
}

pub fn encrypt() -> Result<()> {
    let rsa = generate_keypair()?;

    Ok(())
}
