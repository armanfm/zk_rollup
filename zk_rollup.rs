use warp::Filter;
use serde::{Deserialize, Serialize};
use halo2_proofs::{
    plonk::{create_proof, keygen_pk, keygen_vk, VerifyingKey, SingleVerifier, verify_proof},
    poly::commitment::Params,
    transcript::{Blake2bRead, Blake2bWrite, Challenge255},
};
use pasta_curves::{pallas::{Affine as EpAffine}, vesta::Base as Fq};
use rand_core::OsRng;
use std::convert::TryInto;
use std::fs;
use pasta_curves::group::ff::PrimeField;

#[derive(Deserialize)]
struct AggregatorRequest {
    proof_bytes: Vec<u8>,
    sub_inputs: Vec<Vec<u8>>, // já em bytes
}

#[derive(Serialize)]
struct AggregatorResponse {
    message: String,
    proof_size: usize,
}

async fn aggregate_flow_from_json(mut requests: Vec<AggregatorRequest>) -> Result<String, String> {
    println!("📥 Recebidas {} provas do prover", requests.len());

    // --- Adiciona prova fake para teste de fluxo ---
    let fake_proof = vec![0u8; 32];
    let fake_inputs = vec![vec![Fq::zero(); 2]];
    println!("🔹 Adicionada prova fake para teste de fluxo");
    requests.insert(0, AggregatorRequest { proof_bytes: fake_proof, sub_inputs: vec![vec![0u8;32]; 2] });

    let k = 8;
    let params: Params<EpAffine> = Params::new(k);

    let mut proofs_bytes = Vec::new();
    let mut sub_inputs_fq = Vec::new();
    for (i, req) in requests.iter().enumerate() {
        println!("🔹 Processando prova #{}", i);
        proofs_bytes.push(req.proof_bytes.clone());

        let inputs_fq: Vec<Fq> = req.sub_inputs.iter().map(|b| {
            let arr: [u8; 32] = b.as_slice().try_into().expect("Cada Fq deve ter 32 bytes");
            Fq::from_repr_vartime(arr).expect("Fq inválido")
        }).collect();
        println!("   - Sub-inputs recebidos em Fq: {:?}", inputs_fq);
        sub_inputs_fq.push(inputs_fq);
    }

    // --- Dummy circuit para gerar PK/VK ---
    let dummy_agg = halo2_minimal::aggregator_circuit::AggregatorCircuit {
        sub_proofs: vec![],
        sub_public_inputs: vec![],
        sub_vks: vec![],
        params: params.clone(),
    };
    println!("🔑 Gerando PK/VK do circuito agregador...");
    let vk_agg = keygen_vk(&params, &dummy_agg).map_err(|e| e.to_string())?;
    let pk_agg = keygen_pk(&params, vk_agg.clone(), &dummy_agg).map_err(|e| e.to_string())?;
    println!("✅ PK/VK gerados");

    // --- Circuito real com provas recebidas ---
    let agg_circuit = halo2_minimal::aggregator_circuit::AggregatorCircuit {
        sub_proofs: proofs_bytes,
        sub_public_inputs: sub_inputs_fq,
        sub_vks: vec![vk_agg.clone(); requests.len()],
        params: params.clone(),
    };

    // --- MockProver para validação antes de create_proof ---
    println!("🔍 Rodando MockProver antes do create_proof...");
    let k_mock = k;
    let mock = halo2_proofs::dev::MockProver::run(k_mock, &agg_circuit, vec![]).unwrap();
    if let Err(errors) = mock.verify() {
        eprintln!("❌ MockProver geral falhou! Circuito agregador inconsistente.");
        for err in errors {
            eprintln!(" - {:?}", err);
        }
    } else {
        println!("✅ MockProver passou com sucesso!");
    }

    // --- Gera prova agregada ---
    println!("🛠️ Gerando prova agregada...");
    let mut proof_final = Vec::new();
    let mut transcript = Blake2bWrite::<_, _, Challenge255<EpAffine>>::init(&mut proof_final);
    create_proof(&params, &pk_agg, &[agg_circuit], &[], &mut OsRng, &mut transcript)
        .map_err(|e| e.to_string())?;
    println!("✅ Prova agregada gerada com {} bytes", proof_final.len());

    // --- Verifica a prova agregada ---
    let mut transcript_read = Blake2bRead::<_, _, Challenge255<EpAffine>>::init(&proof_final[..]);
    let strategy = SingleVerifier::new(&params);
    let public_inputs: &[&[&[Fq]]] = &[];
    verify_proof(&params, &vk_agg, strategy, public_inputs, &mut transcript_read)
        .map_err(|e| e.to_string())?;
    println!("✅ Prova agregada verificada com sucesso!");

    fs::write("proof_aggregated.bin", &proof_final).map_err(|e| e.to_string())?;
    println!("💾 Prova agregada salva em proof_aggregated.bin");

    Ok(format!("Prova agregada gerada e verificada! Bytes: {}", proof_final.len()))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let aggregate_route = warp::path("aggregate")
        .and(warp::post())
        .and(warp::body::json::<Vec<AggregatorRequest>>())
        .and_then(|requests: Vec<AggregatorRequest>| async move {
            match aggregate_flow_from_json(requests).await {
                Ok(msg) => Ok::<_, warp::Rejection>(warp::reply::json(&AggregatorResponse {
                    message: msg.clone(),
                    proof_size: msg.len(),
                })),
                Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(&AggregatorResponse {
                    message: format!("Erro: {}", e),
                    proof_size: 0,
                })),
            }
        });

    let status_route = warp::path::end().map(|| warp::reply::json(&serde_json::json!({
        "status": "ok",
        "message": "Servidor agregador rodando"
    })));

    let routes = status_route.or(aggregate_route);

    println!("🚀 Agregador rodando em http://0.0.0.0:8082");
    warp::serve(routes).run(([0, 0, 0, 0], 8082)).await;

    Ok(())
}
