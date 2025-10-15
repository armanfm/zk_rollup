use halo2_minimal::aggregator_circuit::AggregatorCircuit;
use halo2_proofs::plonk::{keygen_pk, keygen_vk, VerifyingKey, ProvingKey};
use halo2_proofs::poly::commitment::Params;
use pasta_curves::pallas::Affine as EpAffine;

/// Gera PK/VK do agregador
pub fn generate_aggregator_keys(params: &Params<EpAffine>) -> (VerifyingKey<EpAffine>, ProvingKey<EpAffine>) {
    let dummy_agg = AggregatorCircuit {
        sub_proofs: vec![],
        sub_public_inputs: vec![],
        sub_vks: vec![],
        params: params.clone(),
    };

    let vk = keygen_vk(params, &dummy_agg).expect("VK do agregador falhou");
    let pk = keygen_pk(params, vk.clone(), &dummy_agg).expect("PK do agregador falhou");
    (vk, pk)
}
/// Gera prova recursiva
pub fn generate_recursive_proof(
    agg: &AggregatorCircuit,
) -> Result<Vec<u8>, Box<dyn StdError>> {
    let vk: VerifyingKey<EqAffine> = keygen_vk(&agg.params, agg)?;
    let pk: ProvingKey<EqAffine> = keygen_pk(&agg.params, vk.clone(), agg)?;

    let mut proof_bytes = Vec::new();
    let mut transcript = Blake2bWrite::<_, _, Challenge255<EqAffine>>::init(&mut proof_bytes);
    create_proof(&agg.params, &pk, &[agg.clone()], &[], &mut OsRng, &mut transcript)?;

    Ok(proof_bytes)
}
