#[test]
fn encrypt_decrypt_roundtrip() {
    sodiumoxide::init().unwrap();
    let (pub_a, sec_a) = sodiumoxide::crypto::box_::gen_keypair();
    let (pub_b, sec_b) = sodiumoxide::crypto::box_::gen_keypair();

    let msg = b"hello pigeon";
    let ct = secure_p2p_msg::crypto::encrypt_message(&sec_a, &pub_b, msg).unwrap();
    let pt = secure_p2p_msg::crypto::decrypt_message(&ct, &pub_a, &sec_b).unwrap();
    assert_eq!(pt, msg);
}

#[test]
fn auth_token_deterministic() {
    let sk = [7u8; 32];
    let pk = [9u8; 32];
    let t1 = secure_p2p_msg::crypto::derive_auth_token(&sk, &pk);
    let t2 = secure_p2p_msg::crypto::derive_auth_token(&sk, &pk);
    assert_eq!(t1, t2);
}


