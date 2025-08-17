use secure_p2p_msg::messaging::message::EnvelopeV1;

#[test]
fn envelope_roundtrip_bincode() {
    let env = EnvelopeV1::new(1, 2, b"hello".to_vec());
    let bytes = bincode::serialize(&env).unwrap();
    let back: EnvelopeV1 = bincode::deserialize(&bytes).unwrap();
    assert_eq!(env, back);
}


