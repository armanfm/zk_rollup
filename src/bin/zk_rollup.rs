use warp::Filter;
use serde::{Deserialize, Serialize};
use halo2_proofs::{
    plonk::{create_proof, keygen_pk, keygen_vk, ProvingKey, VerifyingKey},
    poly::commitment::Params,
    transcript::{Blake2bWrite, Challenge255},
};
use pasta_curves::{vesta::Base as Fq, pallas::Affine as EpAffine};
use rand::thread_rng;
use std::sync::{Arc, Mutex};
use std::fs;
use anyhow::Result;
use halo2_minimal::{AggregatorCircuit, MyFqCircuit};
use reqwest::Client;
use pasta_curves::group::ff::PrimeField;
// --------------------
// Custom Warp Error
// --------------------
#[derive(Debug)]
struct CustomError(String);
impl warp::reject::Reject for CustomError {}

// --------------------
// Request / Response
// --------------------
#[derive(Deserialize)]
struct AggregatorRequest {
    sub_inputs: Vec<[u8; 32]>, // recebemos bytes, convertidos para Fq
}

#[derive(Serialize)]
struct ProofResponse {
    proof: Vec<u8>,
}

// --------------------
// Estado do agregador
// --------------------
struct ProverState {
    pk: Arc<ProvingKey<EpAffine>>,
    vk: Arc<VerifyingKey<EpAffine>>,
    buffer_sub_inputs: Vec<Vec<Fq>>, // armazena as subprovas antes de gerar agregada
}

// --------------------
// Função que gera prova agregada real
// --------------------
fn generate_recursive_proof(
    pk: &ProvingKey<EpAffine>,
    circuit: AggregatorCircuit,
    params: &Params<EpAffine>,
) -> Result<Vec<u8>> {
    println!("🧾 Criando prova agregada real...");
    let mut proof_bytes = Vec::new();
    let mut transcript = Blake2bWrite::<_, _, Challenge255<EpAffine>>::init(&mut proof_bytes);
    let mut rng = thread_rng();

    create_proof(
        params,
        pk,
        &[circuit],
        &[&[]], // instâncias externas vazias
        &mut rng,
        &mut transcript,
    )?;

    println!("✅ Prova agregada criada, tamanho: {} bytes", proof_bytes.len());
    Ok(proof_bytes)
}

// --------------------
// Fluxo de agregação
// --------------------
async fn aggregate_flow(
    requests: Vec<AggregatorRequest>,
    state: Arc<Mutex<ProverState>>,
    k: u32,
) -> Result<Vec<u8>, warp::Rejection> {
    let params: Params<EpAffine> = Params::new(k);

    // Converte bytes recebidos para Fq
    let new_sub_inputs: Vec<Vec<Fq>> = requests
        .iter()
        .map(|req| {
            req.sub_inputs
                .iter()
                .map(|arr| Fq::from_repr_vartime(*arr).expect("bytes inválido para Fq"))
                .collect::<Vec<Fq>>()
        })
        .collect();

    let mut maybe_proof: Option<Vec<u8>> = None;

    {
        let mut guard = state.lock().unwrap();
        // adiciona as subprovas ao buffer
        guard.buffer_sub_inputs.extend(new_sub_inputs);

        // só gera prova agregada quando houver 10 subprovas acumuladas
        if guard.buffer_sub_inputs.len() >= 10 {
            let sub_inputs_to_aggregate = guard.buffer_sub_inputs.drain(..10).collect::<Vec<_>>();
            let (pk_arc, vk_arc) = (guard.pk.clone(), guard.vk.clone());

            let sub_proofs = vec![vec![0u8; 64]; sub_inputs_to_aggregate.len()]; // fake proofs
            let aggregator_circuit = AggregatorCircuit {
                sub_proofs,
                sub_public_inputs: sub_inputs_to_aggregate,
                sub_vks: vec![(*vk_arc).clone(); 10],
                params: params.clone(),
            };

            // gera a prova agregada
            let proof_bytes = generate_recursive_proof(&*pk_arc, aggregator_circuit, &params)
                .map_err(|e| warp::reject::custom(CustomError(format!("{:?}", e))))?;
            maybe_proof = Some(proof_bytes.clone());

            // salva em arquivo
            fs::write("proof_recursive.bin", &proof_bytes)
                .map_err(|e| warp::reject::custom(CustomError(format!("{:?}", e))))?;

            // envia ao Semaphore
            tokio::spawn({
                let proof_to_send = proof_bytes.clone();
                async move {
                    let client = Client::new();
                    let url = "http://127.0.0.1:3030/submit_proof";
                    println!("🚀 Enviando prova agregada ao Semaphore: {}", url);
                    match client.post(url).body(proof_to_send).send().await {
                        Ok(res) => println!("✅ [BG] Semaphore respondeu com status: {}", res.status()),
                        Err(e) => eprintln!("❌ Erro ao enviar ao Semaphore: {:?}", e),
                    }
                }
            });
        }
    }

    maybe_proof.ok_or_else(|| warp::reject::custom(CustomError(
        "Ainda não há 10 subprovas acumuladas, aguardando...".to_string(),
    )))
}

// --------------------
// Warp Server
// --------------------
#[tokio::main]
async fn main() -> Result<()> {
    let k = 8;
    let params: Params<EpAffine> = Params::new(k);

    let dummy_value = Fq::from_raw([0, 0, 0, 0]);
    let dummy_circuit = MyFqCircuit {
        x_fq: Some(dummy_value),
        y_fq: Some(dummy_value),
    };

    let vk = keygen_vk(&params, &dummy_circuit)?;
    let pk = keygen_pk(&params, vk.clone(), &dummy_circuit)?;

    let state: Arc<Mutex<ProverState>> = Arc::new(Mutex::new(ProverState {
        pk: Arc::new(pk),
        vk: Arc::new(vk),
        buffer_sub_inputs: Vec::new(),
    }));

    let state_filter = warp::any().map(move || Arc::clone(&state));

    let aggregate_route = warp::path("aggregate")
        .and(warp::post())
        .and(warp::body::json::<Vec<AggregatorRequest>>())
        .and(state_filter.clone())
        .and_then(move |requests, state| {
            let k_copy = k;
            async move {
                aggregate_flow(requests, state, k_copy).await
                    .map(|proof_bytes| warp::reply::json(&ProofResponse { proof: proof_bytes }))
            }
        });

    let status_route = warp::path::end().map(|| {
        warp::reply::json(&serde_json::json!({
            "status": "ok",
            "message": "Servidor agregador recursivo rodando e conectado ao Semaphore (3030)"
        }))
    });

    println!("🚀 ZK_ROLLUP rodando em http://0.0.0.0:8082");
    warp::serve(status_route.or(aggregate_route)).run(([0, 0, 0, 0], 8082)).await;

    Ok(())
}
