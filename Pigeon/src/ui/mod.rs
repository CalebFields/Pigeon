pub mod cli;

	use std::path::Path;

	/// Resolve destination address and sodium box public key from either a contact selection or explicit args
	#[cfg_attr(not(feature = "network"), allow(dead_code))]
	pub fn resolve_contact_or_args(
		data_dir: &Path,
		contact: Option<&str>,
		to: Option<&str>,
		pubkey_hex: Option<&str>,
	) -> Result<(String, sodiumoxide::crypto::box_::PublicKey), crate::error::Error> {
		if let Some(sel) = contact {
			let store = crate::storage::contacts::ContactStore::open_in_dir(data_dir)
				.map_err(crate::error::Error::Storage)?;
			let found = if let Ok(id) = sel.parse::<u64>() {
				store.get(id).map_err(crate::error::Error::Storage)?
			} else {
				let list = store.list().map_err(crate::error::Error::Storage)?;
				list.into_iter().find(|c| c.name.eq_ignore_ascii_case(sel))
			};
			let c = found.ok_or_else(|| crate::error::Error::Config("contact not found".into()))?;
			let pk = sodiumoxide::crypto::box_::PublicKey::from_slice(&c.public_key)
				.ok_or_else(|| crate::error::Error::Config("contact has invalid pubkey".into()))?;
			Ok((c.addr, pk))
		} else {
			let addr = to.ok_or_else(|| crate::error::Error::Config("--to or --contact required".into()))?;
			let hexpk = pubkey_hex.ok_or_else(|| crate::error::Error::Config("--pubkey_hex or --contact required".into()))?;
			let remote_pk_bytes = hex::decode(hexpk)
				.map_err(|e| crate::error::Error::Config(e.to_string()))?;
			let pk = sodiumoxide::crypto::box_::PublicKey::from_slice(&remote_pk_bytes)
				.ok_or_else(|| crate::error::Error::Config("invalid pubkey".to_string()))?;
			Ok((addr.to_string(), pk))
		}
	}


