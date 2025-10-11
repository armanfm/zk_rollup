use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner},
    plonk::{Circuit, ConstraintSystem, Error, VerifyingKey, verify_proof, SingleVerifier},
    poly::commitment::Params,
    transcript::{Blake2bRead, Challenge255},
};
use pasta_curves::pallas::{Affine as EpAffine, Scalar as Fr};
use std::io::Cursor;

/// Circuito que verifica uma prova externa (do PaiCircuit)
#[derive(Clone)]
pub struct VerifierCircuit {
    pub proof_bytes: Vec<u8>,
    pub public_input: Vec<Fr>,       // múltiplos inputs públicos
    pub vk: VerifyingKey<EpAffine>,
    pub params: Params<EpAffine>,    // params usados pelo PaiCircuit
}

impl Circuit<Fr> for VerifierCircuit {
    type Config = ();
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            proof_bytes: vec![],
            public_input: vec![],
            vk: self.vk.clone(),
            params: self.params.clone(),
        }
    }

    fn configure(_meta: &mut ConstraintSystem<Fr>) -> Self::Config {
        ()
    }

    fn synthesize(
        &self,
        _config: Self::Config,
        mut layouter: impl Layouter<Fr>
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "verify external proof",
            |_region| {
                println!("🔹 Verificando prova externa com input {:?}", self.public_input);

                // Transcript a partir dos bytes da prova
                let mut transcript =
                    Blake2bRead::<_, _, Challenge255<EpAffine>>::init(Cursor::new(&self.proof_bytes));

                // Estratégia de verificação
                let strategy = SingleVerifier::new(&self.params);

                // Formato correto de inputs públicos para Halo2
                let public_inputs_ref: &[&[&[Fr]]] = &[&[&self.public_input]];

                // Verificação real da prova
                verify_proof(
                    &self.params,
                    &self.vk,
                    strategy,
                    public_inputs_ref,
                    &mut transcript,
                ).map_err(|_| Error::Synthesis)?;

                println!("✅ Prova verificada com sucesso!");
                Ok(())
            },
        )
    }
}
