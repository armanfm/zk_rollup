use halo2_minimal::aggregator_circuit::AggregatorCircuit;
use halo2_proofs::plonk::{keygen_pk, keygen_vk, VerifyingKey, ProvingKey, create_proof,Circuit};
use halo2_proofs::poly::commitment::Params;
use halo2_proofs::transcript::{Blake2bWrite, Challenge255};
use pasta_curves::pallas::Affine as EpAffine;
use rand::rngs::OsRng;
use std::error::Error;
use halo2_minimal::MyFqCircuit;

/// Gera PK/VK do agregador
pub fn generate_aggregator_keys(
    params: &Params<EpAffine>
) -> (VerifyingKey<EpAffine>, ProvingKey<EpAffine>) {
    // Dummy circuit sem witnesses
    let dummy_circuit = MyFqCircuit { x_fq: None, y_fq: None }.without_witnesses();

    let vk = keygen_vk(params, &dummy_circuit).expect("VK do agregador falhou");
    let pk = keygen_pk(params, vk.clone(), &dummy_circuit).expect("PK do agregador falhou");

    (vk, pk)
}

/// Gera prova recursiva
pub fn generate_recursive_proof(
    agg: &AggregatorCircuit,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let vk: VerifyingKey<EpAffine> = keygen_vk(&agg.params, agg)?;
    let pk: ProvingKey<EpAffine> = keygen_pk(&agg.params, vk.clone(), agg)?;

    let mut proof_bytes = Vec::new();
    let mut transcript = Blake2bWrite::<_, _, Challenge255<EpAffine>>::init(&mut proof_bytes);

    let mut rng = OsRng;
    create_proof(&agg.params, &pk, &[agg.clone()], &[], &mut rng, &mut transcript)?;

    Ok(transcript.finalize().to_vec())
}

/// Função main do binário
fn main() -> Result<(), Box<dyn Error>> {
    let k = 8;
    let params = Params::<EpAffine>::new(k);

    // Cria circuito do agregador (dummy)
    let agg = AggregatorCircuit {
        sub_proofs: vec![],
        sub_public_inputs: vec![],
        sub_vks: vec![],
        params: params.clone(),
    };

    println!("⚙️ Gerando PK e VK do agregador...");
    let (_vk, _pk) = generate_aggregator_keys(&params);
    println!("✅ PK e VK gerados.");

    println!("🧾 Criando prova recursiva...");
    let proof = generate_recursive_proof(&agg)?;
    println!("✅ Prova recursiva criada, {} bytes", proof.len());

    Ok(())
}
