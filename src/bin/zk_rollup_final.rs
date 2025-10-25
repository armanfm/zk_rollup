use warp::Filter;
use std::sync::Arc;
use tokio::sync::Mutex;
use halo2_proofs::{
    plonk::{keygen_vk, VerifyingKey},
    poly::commitment::Params,
};
use halo2curves::pasta::pallas::Affine as EqAffine;
use halo2_minimal::AggregatorEqCircuit;
use serde::Serialize;
use bytes::Bytes;
use pasta_curves::pallas::Scalar as Fr;
use std::fs;

// ----------------- Helpers -----------------
#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    message: String,
}

// ----------------- Semaphore -----------------
pub struct Semaphore {
    pub params: Params<EqAffine>,
    pub buffer_proofs: Vec<Vec<u8>>,
    pub buffer_vks: Vec<VerifyingKey<EqAffine>>,
}

impl Semaphore {
    pub fn new(params: Params<EqAffine>) -> Self {
        Self {
            params,
            buffer_proofs: vec![],
            buffer_vks: vec![],
        }
    }

    /// Recebe uma prova agregada e cria AggregatorEqCircuit quando houver 3
    pub fn submit_proof(
        &mut self,
        proof_bytes: Vec<u8>,
        vk: VerifyingKey<EqAffine>,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        println!("🔹 Recebida prova agregada de {} bytes", proof_bytes.len());
        self.buffer_proofs.push(proof_bytes);
        self.buffer_vks.push(vk);

        println!("📥 Buffer atual: {} prova(s)", self.buffer_proofs.len());

        if self.buffer_proofs.len() < 3 {
            println!("⏳ Ainda não há 3 provas agregadas, aguardando...");
            return Ok(None);
        }

        println!("🚀 3 provas agregadas recebidas! Criando AggregatorEqCircuit nível superior...");

        // Pega as 3 provas e os 3 VKs
        let sub_proofs = self.buffer_proofs.drain(..3).collect::<Vec<_>>();
        let sub_vks = self.buffer_vks.drain(..3).collect::<Vec<_>>();

        // Preenche sub_public_inputs com placeholders
        let sub_public_inputs = vec![vec![Fr::zero()]; sub_proofs.len()];

        let agg_circuit = AggregatorEqCircuit {
            sub_proofs: sub_proofs.clone(),
            sub_public_inputs,
            sub_vks,
            params: self.params.clone(),
        };

        let _vk_level_up: VerifyingKey<EqAffine> = keygen_vk(&self.params, &agg_circuit)?;

        println!("✅ AggregatorEqCircuit nível superior criado com sucesso!");

        // Concatena os bytes das 3 provas como placeholder da agregação
        let aggregated_bytes = agg_circuit.sub_proofs.concat();

        // Mostra apenas o tamanho dos bytes
        println!(
            "📦 Tamanho da agregação nível superior: {} bytes",
            aggregated_bytes.len()
        );

        // Salva em arquivo
        fs::write("aggregated_proof_level_up.bin", &aggregated_bytes)?;
        println!("✅ Prova nível superior salva em aggregated_proof_level_up.bin");

        Ok(Some(aggregated_bytes))
    }
}

// ----------------- Warp helpers -----------------
fn with_semaphore(
    semaphore: Arc<Mutex<Semaphore>>,
) -> impl Filter<Extract = (Arc<Mutex<Semaphore>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || semaphore.clone())
}

async fn handle_verify(
    proof_bytes: Bytes,
    semaphore: Arc<Mutex<Semaphore>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut sem = semaphore.lock().await;

    // Dummy VK para cada prova agregada recebida
    let dummy_vk = keygen_vk(&sem.params, &AggregatorEqCircuit {
        sub_proofs: vec![],
        sub_public_inputs: vec![],
        sub_vks: vec![],
        params: sem.params.clone(),
    }).unwrap();

    match sem.submit_proof(proof_bytes.to_vec(), dummy_vk) {
        Ok(Some(aggregated_proof)) => Ok(warp::reply::json(&ApiResponse {
            success: true,
            message: format!("Aggregação nível superior gerada, {} bytes", aggregated_proof.len()),
        })),
        Ok(None) => Ok(warp::reply::json(&ApiResponse {
            success: true,
            message: "Prova recebida, aguardando mais provas agregadas".to_string(),
        })),
        Err(e) => Ok(warp::reply::json(&ApiResponse {
            success: false,
            message: format!("Erro ao processar prova: {:?}", e),
        })),
    }
}

// ----------------- Main -----------------
#[tokio::main]
async fn main() {
    let k = 8u32;
    let params: Params<EqAffine> = Params::new(k);

    let semaphore = Arc::new(Mutex::new(Semaphore::new(params)));

    let route = warp::post()
        .and(warp::path("submit_proof"))
        .and(warp::body::bytes())
        .and(with_semaphore(semaphore.clone()))
        .and_then(handle_verify);

    println!("🚀 Servidor rodando em http://127.0.0.1:3030");
    warp::serve(route).run(([127, 0, 0, 1], 3030)).await;
}

