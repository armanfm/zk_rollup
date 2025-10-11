use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Circuit, ConstraintSystem, Error, VerifyingKey},
};
use pasta_curves::pallas::{Affine as EpAffine, Scalar as Fr};

/// Circuito que representa a verificação de uma prova externa.
#[derive(Clone)]
pub struct VerifierCircuit {
    pub proof_bytes: Vec<u8>,
    pub public_input: Fr,
    pub vk: VerifyingKey<EpAffine>,
}

impl Circuit<Fr> for VerifierCircuit {
    type Config = ();
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            proof_bytes: vec![],
            public_input: Fr::zero(),
            vk: self.vk.clone(),
        }
    }

    fn configure(_meta: &mut ConstraintSystem<Fr>) -> Self::Config {
        ()
    }

    fn synthesize(
        &self,
        _config: Self::Config,
        mut layouter: impl Layouter<Fr>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "verify external proof",
            |_region| {
                // 🔹 Aqui seria onde o gadget real de verificação acontece.
                // Por enquanto, apenas simulamos a checagem:
                println!("Verificando prova de {} bytes com input {:?}", 
                    self.proof_bytes.len(), self.public_input);

                // Aqui seria o ponto de substituir por algo como:
                // let ok = ProofVerifierGadget::verify(&self.vk, &self.proof_bytes, &[self.public_input])?;
                // region.constrain_equal(ok, Value::known(Fr::one()));

                Ok(())
            },
        )
    }
}
