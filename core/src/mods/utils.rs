/*
 * This code contains proprietary source code from NEHONIX
 * Copyright © 2025 NEHONIX - www.nehonix.com
 * Licensed under NEHONIX Open Source License (NOSL) v1.0
 */

use sha2::{Sha256, Digest};

/// Generates a SHA-256 hash of the domain name and returns it as a hex string.
pub fn get_domain_hash(domain: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(domain.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}
