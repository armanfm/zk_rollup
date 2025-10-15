use halo2_proofs::{
    arithmetic::Field,
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Selector, Error},
};
use pasta_curves::vesta::Base as Fq;

#[derive(Clone, Debug)]
pub struct RecursiveConfig {
    pub input: Column<Advice>,
    pub output: Column<Advice>,
    pub s_gate: Selector,
}

#[derive(Clone)]
pub struct RecursiveCircuit {
    pub values: Vec<Fq>, // valores já verificados das provas
}

impl Circuit<Fq> for RecursiveCircuit {
    type Config = RecursiveConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self { values: vec![] }
    }

    fn configure(meta: &mut ConstraintSystem<Fq>) -> Self::Config {
        let input = meta.advice_column();
        let output = meta.advice_column();
        let s_gate = meta.selector();

        meta.create_gate("aggregate", |meta| {
            let s = meta.query_selector(s_gate);
            let input_val = meta.query_advice(input, halo2_proofs::poly::Rotation::cur());
            let output_val = meta.query_advice(output, halo2_proofs::poly::Rotation::cur());
            vec![s * (output_val - input_val)] // gate simplificada
        });

        RecursiveConfig { input, output, s_gate }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fq>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "recursive region",
            |mut region| {
                for (i, val) in self.values.iter().enumerate() {
                    config.s_gate.enable(&mut region, i)?;
                    region.assign_advice(|| "input", config.input, i, || Value::known(*val))?;
                    region.assign_advice(|| "output", config.output, i, || Value::known(*val))?;
                }
                Ok(())
            },
        )
    }
}
